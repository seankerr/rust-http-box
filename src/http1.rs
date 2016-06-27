// +-----------------------------------------------------------------------------------------------+
// | Copyright 2016 Sean Kerr                                                                      |
// |                                                                                               |
// | Licensed under the Apache License, Version 2.0 (the "License");                               |
// | you may not use this file except in compliance with the License.                              |
// | You may obtain a copy of the License at                                                       |
// |                                                                                               |
// |  http://www.apache.org/licenses/LICENSE-2.0                                                   |
// |                                                                                               |
// | Unless required by applicable law or agreed to in writing, software                           |
// | distributed under the License is distributed on an "AS IS" BASIS,                             |
// | WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.                      |
// | See the License for the specific language governing permissions and                           |
// | limitations under the License.                                                                |
// +-----------------------------------------------------------------------------------------------+
// | Author: Sean Kerr <sean@code-box.org>                                                         |
// +-----------------------------------------------------------------------------------------------+

//! HTTP 1.1 parser, states, and errors.

#![allow(dead_code)]

use byte::hex_to_byte;
use byte::is_token;
use fsm::{ ParserValue,
           Success };

use std::{ fmt,
           str };
use std::borrow;
use std::hash;

// -------------------------------------------------------------------------------------------------

/// State flag mask.
const FLAG_MASK: u32 = 0xF;

/// State flag shift.
const FLAG_SHIFT: u8 = 0;

/// Lower 14 bits mask.
const LOWER14_MASK: u32 = 0x3FFF;

/// Lower 14 bits shift.
const LOWER14_SHIFT: u8 = 4;

/// Upper 14 bits mask.
const UPPER14_MASK: u32 = 0x3FFF;

/// Upper 14 bits shift.
const UPPER14_SHIFT: u8 = 18;

// -------------------------------------------------------------------------------------------------
// FLAGS
// -------------------------------------------------------------------------------------------------

/// Parsing chunk encoded data.
const F_CHUNKED: u32 = 1;

/// Parsing chunk encoded extensions.
const F_CHUNK_EXTENSIONS: u32 = 2;

/// Headers are finished parsing.
const F_HEADERS_FINISHED: u32 = 4;

/// Parsing multipart data.
const F_MULTIPART: u32 = 8;

// -------------------------------------------------------------------------------------------------
// BIT DATA MACROS
// -------------------------------------------------------------------------------------------------

/// Retrieve the lower 14 bits.
macro_rules! get_lower14 {
    ($parser:expr) => ({
        ($parser.bit_data >> LOWER14_SHIFT) & LOWER14_MASK
    });
}

/// Retrieve the upper 14 bits.
macro_rules! get_upper14 {
    ($parser:expr) => ({
        ($parser.bit_data >> UPPER14_SHIFT) & UPPER14_MASK
    });
}

/// Indicates that a state flag is set.
macro_rules! has_flag {
    ($parser:expr, $flag:expr) => ({
        (($parser.bit_data >> FLAG_SHIFT) & FLAG_MASK) & $flag == $flag
    });
}

/// Set a state flag.
macro_rules! set_flag {
    ($parser:expr, $flag:expr) => ({
        $parser.bit_data |= ($flag & FLAG_MASK) << FLAG_SHIFT;
    });
}

/// Set the lower 14 bits.
macro_rules! set_lower14 {
    ($parser:expr, $bits:expr) => ({
        let bits = $bits as u32;

        $parser.bit_data &= !(LOWER14_MASK << LOWER14_SHIFT);
        $parser.bit_data |= bits << LOWER14_SHIFT;
    });
}

/// Set the upper 14 bits.
macro_rules! set_upper14 {
    ($parser:expr, $bits:expr) => ({
        let bits = $bits as u32;

        $parser.bit_data &= !(UPPER14_MASK << UPPER14_SHIFT);
        $parser.bit_data |= bits << UPPER14_SHIFT;
    });
}

/// Unset a state flag.
macro_rules! unset_flag {
    ($parser:expr, $flag:expr) => ({
        $parser.bit_data &= !(($flag & FLAG_MASK) << FLAG_SHIFT);
    });
}

// -------------------------------------------------------------------------------------------------

/// Parser state function type.
type StateFunction<'a, T> = fn(&mut Parser<'a, T>, &mut ParserContext<T>)
    -> Result<ParserValue, ParserError>;

// -------------------------------------------------------------------------------------------------

/// HTTP cookie.
#[derive(Clone,Eq,PartialEq)]
pub struct Cookie {
    /// Domain.
    domain: Option<String>,

    /// Expiration date and time.
    expires: Option<String>,

    /// Indicates the cookie is for HTTP only.
    http_only: bool,

    /// Maximum age.
    max_age: Option<String>,

    /// Name.
    name: String,

    /// Path.
    path: Option<String>,

    /// Indicates that the cookie is secure.
    secure: bool,

    /// Value.
    value: Option<String>
}

impl Cookie {
    /// Create a new `Cookie`.
    pub fn new(name: &str) -> Cookie {
        Cookie{
            domain:    None,
            expires:   None,
            http_only: false,
            max_age:   None,
            name:      name.to_string(),
            path:      None,
            secure:    false,
            value:     None
        }
    }

    /// Create a new `Cookie`.
    pub fn new_from_slice(name: &[u8]) -> Cookie {
        Cookie{
            domain:    None,
            expires:   None,
            http_only: false,
            max_age:   None,
            name:      unsafe {
                let mut s = String::with_capacity(name.len());

                s.as_mut_vec().extend_from_slice(name);
                s
            },
            path:      None,
            secure:    false,
            value:     None
        }
    }

    /// Retrieve the domain.
    pub fn get_domain(&self) -> Option<&str> {
        if let Some(ref x) = self.domain {
            Some(x)
        } else {
            None
        }
    }

    /// Retrieve the expiration date and time.
    pub fn get_expires(&self) -> Option<&str> {
        if let Some(ref x) = self.expires {
            Some(x)
        } else {
            None
        }
    }

    /// Retrieve the maximum age.
    pub fn get_max_age(&self) -> Option<&str> {
        if let Some(ref x) = self.max_age {
            Some(x)
        } else {
            None
        }
    }

    /// Retrieve the name.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Retrieve the path.
    pub fn get_path(&self) -> Option<&str> {
        if let Some(ref x) = self.path {
            Some(x)
        } else {
            None
        }
    }

    /// Retrieve the value.
    pub fn get_value(&self) -> Option<&str> {
        if let Some(ref x) = self.value {
            Some(x)
        } else {
            None
        }
    }

    /// Indicates that the cookie is for HTTP only.
    pub fn is_http_only(&self) -> bool {
        self.http_only
    }

    /// Indicates that the cookie is secure.
    pub fn is_secure(&self) -> bool {
        self.secure
    }

    /// Set the domain.
    pub fn set_domain(&mut self, domain: &str) -> &mut Self {
        self.domain = Some(domain.to_string());
        self
    }

    /// Set the domain.
    pub fn set_domain_from_slice(&mut self, domain: &[u8]) -> &mut Self {
        self.domain = Some(unsafe {
            let mut s = String::with_capacity(domain.len());

            s.as_mut_vec().extend_from_slice(domain);
            s
        });

        self
    }

    /// Set the expiration date and time.
    pub fn set_expires(&mut self, expires: &str) -> &mut Self {
        self.expires = Some(expires.to_string());
        self
    }

    /// Set the expiration date and time.
    pub fn set_expires_from_slice(&mut self, expires: &[u8]) -> &mut Self {
        self.expires = Some(unsafe {
            let mut s = String::with_capacity(expires.len());

            s.as_mut_vec().extend_from_slice(expires);
            s
        });

        self
    }

    /// Set the HTTP only status.
    pub fn set_http_only(&mut self, http_only: bool) -> &mut Self {
        self.http_only = http_only;
        self
    }

    /// Set the maximum age.
    pub fn set_max_age(&mut self, max_age: &str) -> &mut Self {
        self.max_age = Some(max_age.to_string());
        self
    }

    /// Set the maximum age.
    pub fn set_max_age_from_slice(&mut self, max_age: &[u8]) -> &mut Self {
        self.max_age = Some(unsafe {
            let mut s = String::with_capacity(max_age.len());

            s.as_mut_vec().extend_from_slice(max_age);
            s
        });

        self
    }

    /// Set the name.
    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = name.to_string();
        self
    }

    /// Set the name.
    pub fn set_name_from_slice(&mut self, name: &[u8]) -> &mut Self {
        self.name = unsafe {
            let mut s = String::with_capacity(name.len());

            s.as_mut_vec().extend_from_slice(name);
            s
        };

        self
    }

    /// Set the path.
    pub fn set_path(&mut self, path: &str) -> &mut Self {
        self.path = Some(path.to_string());
        self
    }

    /// Set the path.
    pub fn set_path_from_slice(&mut self, path: &[u8]) -> &mut Self {
        self.path = Some(unsafe {
            let mut s = String::with_capacity(path.len());

            s.as_mut_vec().extend_from_slice(path);
            s
        });

        self
    }

    /// Set the secure status.
    pub fn set_secure(&mut self, secure: bool) -> &mut Self {
        self.secure = secure;
        self
    }

    /// Set the value.
    pub fn set_value(&mut self, value: &str) -> &mut Self {
        self.value = Some(value.to_string());
        self
    }

    /// Set the value.
    pub fn set_value_from_slice(&mut self, value: &[u8]) -> &mut Self {
        self.value = Some(unsafe {
            let mut s = String::with_capacity(value.len());

            s.as_mut_vec().extend_from_slice(value);
            s
        });

        self
    }
}

impl borrow::Borrow<str> for Cookie {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl PartialEq<str> for Cookie {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.name == other
    }
}

impl fmt::Debug for Cookie {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter,
               "Cookie(name=\"{}\", value=\"{}\", domain=\"{}\", path=\"{}\", \
                       expires=\"{}\", max-age=\"{}\", http-only={}, secure={})",
               self.name,
               self.value.clone().unwrap_or("".to_string()),
               self.domain.clone().unwrap_or("".to_string()),
               self.path.clone().unwrap_or("".to_string()),
               self.expires.clone().unwrap_or("".to_string()),
               self.max_age.clone().unwrap_or("".to_string()),
               self.http_only,
               self.secure)
    }
}

impl fmt::Display for Cookie {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.value.clone().unwrap_or("".to_string()))
    }
}

impl hash::Hash for Cookie {
    #[inline]
    fn hash<H>(&self, state: &mut H) where H : hash::Hasher {
        self.name.hash(state)
    }
}

// -------------------------------------------------------------------------------------------------

/// Parser error messages.
#[derive(Clone,Copy,PartialEq)]
pub enum ParserError {
    /// Invalid chunk extension name on byte `u8`.
    ChunkExtensionName(u8),

    /// Invalid chunk extension value on byte `u8`.
    ChunkExtensionValue(u8),

    /// Invalid chunk length on byte `u8`.
    ChunkLength(u8),

    /// Invalid CRLF sequence on byte `u8`.
    CrlfSequence(u8),

    /// Parsing has failed.
    Dead,

    /// Invalid header field on byte `u8`.
    HeaderField(u8),

    /// Invalid header value on byte `u8`.
    HeaderValue(u8),

    /// Maximum chunk length has been met.
    MaxChunkLength,

    /// Maximum headers length has been met.
    MaxHeadersLength,

    /// Invalid request method on byte `u8`.
    Method(u8),

    /// Invalid multipart boundary on byte `u8`.
    MultipartBoundary(u8),

    /// Invalid status on byte `u8`.
    Status(u8),

    /// Invalid status code on byte `u8`.
    StatusCode(u8),

    /// Invalid URL character on byte `u8`.
    Url(u8),

    /// Invalid URL encoded field on byte `u8`.
    UrlEncodedField(u8),

    /// Invalid URL encoded value on byte `u8`.
    UrlEncodedValue(u8),

    /// Invalid HTTP version on byte `u8`.
    Version(u8),
}

impl fmt::Debug for ParserError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserError::ChunkExtensionName(ref byte) => {
                write!(formatter, "ParserError::ChunkExtensionName(Invalid chunk extension name on byte {})",
                       byte)
            },
            ParserError::ChunkExtensionValue(ref byte) => {
                write!(formatter, "ParserError::ChunkExtensionValue(Invalid chunk extension value on byte {})",
                       byte)
            },
            ParserError::ChunkLength(ref byte) => {
                write!(formatter, "ParserError::ChunkLength(Invalid chunk length on byte {})", byte)
            },
            ParserError::CrlfSequence(ref byte) => {
                write!(formatter, "ParserError::CrlfSequence(Invalid CRLF sequence on byte {})", byte)
            },
            ParserError::Dead => {
                write!(formatter, "ParserError::Dead(Parser is dead)")
            },
            ParserError::HeaderField(ref byte) => {
                write!(formatter, "ParserError::HeaderField(Invalid header field on byte {})", byte)
            },
            ParserError::HeaderValue(ref byte) => {
                write!(formatter, "ParserError::HeaderValue(Invalid header value on byte {})", byte)
            },
            ParserError::MaxChunkLength => {
                write!(formatter, "ParserError::MaxChunkLength(Maximum chunk length has been met)")
            },
            ParserError::MaxHeadersLength => {
                write!(formatter, "ParserError::MaxHeadersLength(Maximum headers length has been met)")
            },
            ParserError::Method(ref byte) => {
                write!(formatter, "ParserError::Method(Invalid method on byte {})", byte)
            },
            ParserError::MultipartBoundary(ref byte) => {
                write!(formatter, "ParserError::MultipartBoundary(Invalid multipart boundary on byte {})",
                       byte)
            },
            ParserError::Status(ref byte) => {
                write!(formatter, "ParserError::Status(Invalid status on byte {})", byte)
            },
            ParserError::StatusCode(ref byte) => {
                write!(formatter, "ParserError::StatusCode(Invalid status code on byte {})", byte)
            },
            ParserError::Url(ref byte) => {
                write!(formatter, "ParserError::Url(Invalid URL on byte {})", byte)
            },
            ParserError::UrlEncodedField(ref byte) => {
                write!(formatter, "ParserError::UrlEncodedField(Invalid URL encoded field on byte {})",
                       byte)
            },
            ParserError::UrlEncodedValue(ref byte) => {
                write!(formatter, "ParserError::UrlEncodedValue(Invalid URL encoded value on byte {})",
                       byte)
            },
            ParserError::Version(ref byte) => {
                write!(formatter, "ParserError::Version(Invalid HTTP version on byte {})", byte)
            }
        }
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserError::ChunkExtensionName(ref byte) => {
                write!(formatter, "Invalid chunk extension name on byte {}", byte)
            },
            ParserError::ChunkExtensionValue(ref byte) => {
                write!(formatter, "Invalid chunk extension value on byte {}", byte)
            },
            ParserError::ChunkLength(ref byte) => {
                write!(formatter, "Invalid chunk length on byte {}", byte)
            },
            ParserError::CrlfSequence(ref byte) => {
                write!(formatter, "Invalid CRLF sequence on byte {}", byte)
            },
            ParserError::Dead => {
                write!(formatter, "Parser is dead")
            },
            ParserError::HeaderField(ref byte) => {
                write!(formatter, "Invalid header field on byte {}", byte)
            },
            ParserError::HeaderValue(ref byte) => {
                write!(formatter, "Invalid header value on byte {}", byte)
            },
            ParserError::MaxChunkLength => {
                write!(formatter, "Maximum chunk length has been met")
            },
            ParserError::MaxHeadersLength => {
                write!(formatter, "Maximum headers length has been met")
            },
            ParserError::Method(ref byte) => {
                write!(formatter, "Invalid method on byte {}", byte)
            },
            ParserError::MultipartBoundary(ref byte) => {
                write!(formatter, "Invalid multipart boundary on byte {}", byte)
            },
            ParserError::Status(ref byte) => {
                write!(formatter, "Invalid status on byte {}", byte)
            },
            ParserError::StatusCode(ref byte) => {
                write!(formatter, "Invalid status code on byte {}", byte)
            },
            ParserError::Url(ref byte) => {
                write!(formatter, "Invalid URL on byte {}", byte)
            },
            ParserError::UrlEncodedField(ref byte) => {
                write!(formatter, "Invalid URL encoded field on byte {}", byte)
            },
            ParserError::UrlEncodedValue(ref byte) => {
                write!(formatter, "Invalid URL encoded value on byte {}", byte)
            },
            ParserError::Version(ref byte) => {
                write!(formatter, "Invalid HTTP version on byte {}", byte)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Parser types.
///
/// The parser type will be `ParserType::Unknown` until
/// [`Parser::parse_headers()`](struct.Parser.html#method.parse_headers)
/// is executed, and either of the following has occurred:
///
/// *Requests:*
///
/// [`Http1Handler::on_method()`](trait.Http1Handler.html#method.on_method) has been executed.
///
/// *Responses:*
///
/// [`Http1Handler::on_version()`](trait.Http1Handler.html#method.on_version) has been executed.
#[derive(Clone,Copy)]
pub enum ParserType {
    /// Parser is parsing a request.
    Request,

    /// Parser is parsing a response.
    Response,

    /// Type has not yet been determined.
    Unknown
}

impl fmt::Debug for ParserType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserType::Request => {
                write!(formatter, "ParserType::Request")
            },
            ParserType::Response => {
                write!(formatter, "ParserType::Response")
            },
            ParserType::Unknown => {
                write!(formatter, "ParserType::Unknown")
            }
        }
    }
}

impl fmt::Display for ParserType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserType::Request => {
                write!(formatter, "ParserType::Request")
            },
            ParserType::Response => {
                write!(formatter, "ParserType::Response")
            },
            ParserType::Unknown => {
                write!(formatter, "ParserType::Unknown")
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Parser states.
///
/// These states are in an order that can be compared so the progress can be checked by the parser
/// functions.
#[derive(Clone,Copy,Debug,PartialEq)]
#[repr(u8)]
pub enum ParserState {
    /// An error was returned from a call to `Parser::parse()`.
    Dead,

    /// Stripping linear white space before request/response detection.
    StripDetect,

    /// Detect request/response byte 1.
    Detect1,

    /// Detect request/response byte 2.
    Detect2,

    /// Detect request/response byte 3.
    Detect3,

    /// Detect request/response byte 4.
    Detect4,

    /// Detect request/response byte 5.
    Detect5,

    // ---------------------------------------------------------------------------------------------
    // REQUEST
    // ---------------------------------------------------------------------------------------------

    /// Parsing request method.
    RequestMethod,

    /// Stripping linear white space before URL.
    StripRequestUrl,

    /// Parsing URL.
    RequestUrl,

    /// Stripping linear white space before request HTTP version.
    StripRequestHttp,

    /// Parsing request HTTP version byte 1.
    RequestHttp1,

    /// Parsing request HTTP version byte 2.
    RequestHttp2,

    /// Parsing request HTTP version byte 3.
    RequestHttp3,

    /// Parsing request HTTP version byte 4.
    RequestHttp4,

    /// Parsing request HTTP version byte 5.
    RequestHttp5,

    /// Parsing request HTTP major version.
    RequestVersionMajor,

    /// Parsing request HTTP minor version.
    RequestVersionMinor,

    // ---------------------------------------------------------------------------------------------
    // RESPONSE
    // ---------------------------------------------------------------------------------------------

    /// Parsing response HTTP major version.
    ResponseVersionMajor,

    /// Parsing response HTTP minor version.
    ResponseVersionMinor,

    /// Stripping linear white space before response status code.
    StripResponseStatusCode,

    /// Parsing response status code.
    ResponseStatusCode,

    /// Stripping linear white space before response status.
    StripResponseStatus,

    /// Parsing response status.
    ResponseStatus,

    // ---------------------------------------------------------------------------------------------
    // HEADERS
    // ---------------------------------------------------------------------------------------------

    // pre-header states:
    //   These only exist purely to avoid the situation where a client can send an initial
    //   request/response line then CRLF[SPACE], and the parser would have assumed the next
    //   piece of content is the second line of a multiline header value.
    //
    //   In addition to this, multiline header value support has been deprecated, but we'll keep
    //   support for now: https://tools.ietf.org/html/rfc7230#section-3.2.4

    /// Parsing pre-header line feed.
    PreHeaders1,

    /// Parsing pre-header potential carriage return.
    PreHeaders2,

    /// Stripping linear white space before header field.
    StripHeaderField,

    /// Parsing first byte of header field.
    FirstHeaderField,

    /// Parsing upper-cased header field.
    UpperHeaderField,

    /// Parsing lower-cased header field.
    LowerHeaderField,

    /// Stripping linear white space before header value.
    StripHeaderValue,

    /// Parsing header value.
    HeaderValue,

    /// Parsing quoted header value.
    HeaderQuotedValue,

    /// Parsing escaped header value.
    HeaderEscapedValue,

    /// Parsing first carriage return after header value.
    Newline1,

    /// Parsing first line feed after header value.
    Newline2,

    /// Parsing second carriage return after header value.
    Newline3,

    /// Parsing second line feed after header value.
    Newline4,

    // ---------------------------------------------------------------------------------------------
    // CHUNKED TRANSFER
    // ---------------------------------------------------------------------------------------------

    /// Parsing chunk length byte 1.
    ChunkLength1,

    /// Parsing chunk length byte 2.
    ChunkLength2,

    /// Parsing chunk length end (when chunk length is 0).
    ChunkLengthEnd,

    /// Stripping linear white space before chunk extension name.
    StripChunkExtensionName,

    /// Parsing upper-cased chunk extension.
    UpperChunkExtensionName,

    /// Parsing lower-cased chunk extension.
    LowerChunkExtensionName,

    /// Parsing chunk extension with no value.
    ChunkExtensionNameNoValue,

    /// Stripping linear white space before chunk extension value.
    StripChunkExtensionValue,

    /// Parsing chunk extension value.
    ChunkExtensionValue,

    /// Parsing quoted chunk extension value.
    ChunkExtensionQuotedValue,

    /// Parsing escaped chunk extension value.
    ChunkExtensionEscapedValue,

    /// Parsing potential semi-colon or carriage return after chunk extension quoted value.
    ChunkExtensionSemiColon,

    /// Parsing line feed after chunk length.
    ChunkLengthNewline,

    /// Parsing chunk data.
    ChunkData,

    /// Parsing carriage return after chunk data.
    ChunkDataNewline1,

    /// Parsing line feed after chunk data.
    ChunkDataNewline2,

    // ---------------------------------------------------------------------------------------------
    // MULTIPART
    // ---------------------------------------------------------------------------------------------

    /// Parsing first hyphen before multipart boundary.
    MultipartHyphen1,

    /// Parsing second hyphen before multipart boundary.
    MultipartHyphen2,

    /// Parsing potential first hyphen before multipart boundary.
    MultipartTryHyphen1,

    /// Parsing potential second hyphen before multipart boundary.
    MultipartTryHyphen2,

    /// Parsing potential multipart boundary.
    MultipartTryBoundary,

    /// Parsing carriage return after multipart boundary.
    MultipartNewline1,

    /// Parsing line feed after multipart boundary.
    MultipartNewline2,

    /// Parsing multipart data.
    MultipartData,

    // ---------------------------------------------------------------------------------------------
    // URL ENCODED
    // ---------------------------------------------------------------------------------------------

    /// Parsing URL encoded field.
    UrlEncodedField,

    /// Parsing URL encoded field ampersand.
    UrlEncodedFieldAmpersand,

    /// Parsing URL encoded field hex sequence.
    UrlEncodedFieldHex,

    /// Parsing URL encoded field plus sign.
    UrlEncodedFieldPlus,

    /// Parsing URL encoded value.
    UrlEncodedValue,

    /// Parsing URL encoded value hex sequence.
    UrlEncodedValueHex,

    /// Parsing URL encoded value plus sign.
    UrlEncodedValuePlus,

    // ---------------------------------------------------------------------------------------------
    // FINISHED
    // ---------------------------------------------------------------------------------------------

    /// End of body parsing.
    BodyFinished,

    /// Parsing entire message has finished.
    Finished
}

// -------------------------------------------------------------------------------------------------

/// Type that handles HTTP/1.1 parser events.
#[allow(unused_variables)]
pub trait Http1Handler {
    /// Callback that is executed when body parsing has completed successfully.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_chunked()`](../http1/struct.Parser.html#method.parse_chunked)
    ///
    /// Once all chunked data has been parsed.
    ///
    /// [`Parser::parse_multipart()`](../http1/struct.Parser.html#method.parse_multipart)
    ///
    /// Once all multipart data has been parsed.
    ///
    /// [`Parser::parse_url_encoded()`](../http1/struct.Parser.html#method.parse_url_encoded)
    ///
    /// Once all URL encoded data has been parsed.
    fn on_body_finished(&mut self) -> bool {
        true
    }

    /// Retrieve the multipart boundary.
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_multipart()`](../http1/struct.Parser.html#method.parse_multipart)
    fn get_boundary(&mut self) -> Option<&[u8]> {
        None
    }

    /// Callback that is executed when chunk encoded data has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_chunked()`](../http1/struct.Parser.html#method.parse_chunked)
    fn on_chunk_data(&mut self, data: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a chunk extension name has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_chunked()`](../http1/struct.Parser.html#method.parse_chunked)
    fn on_chunk_extension_name(&mut self, name: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a chunk extension value has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_chunked()`](../http1/struct.Parser.html#method.parse_chunked)
    fn on_chunk_extension_value(&mut self, value: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when chunk extension parsing has completed successfully.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_chunked()`](../http1/struct.Parser.html#method.parse_chunked)
    fn on_chunk_extensions_finished(&mut self) -> bool {
        true
    }

    /// Callback that is executed when a chunk length has been located.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_chunked()`](../http1/struct.Parser.html#method.parse_chunked)
    fn on_chunk_length(&mut self, size: usize) -> bool {
        true
    }

    /// Callback that is executed when a header field has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_chunked()`](../http1/struct.Parser.html#method.parse_chunked)
    ///
    /// If trailers are present.
    ///
    /// [`Parser::parse_headers()`](../http1/struct.Parser.html#method.parse_headers)
    ///
    /// For standard HTTP headers.
    ///
    /// [`Parser::parse_multipart()`](../http1/struct.Parser.html#method.parse_multipart)
    ///
    /// For headers before each multipart section.
    fn on_header_field(&mut self, field: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a header value has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_chunked()`](../http1/struct.Parser.html#method.parse_chunked)
    ///
    /// If trailers are supplied.
    ///
    /// [`Parser::parse_headers()`](../http1/struct.Parser.html#method.parse_headers)
    ///
    /// For standard HTTP headers.
    ///
    /// [`Parser::parse_multipart()`](../http1/struct.Parser.html#method.parse_multipart)
    ///
    /// For headers before each multipart section.
    fn on_header_value(&mut self, value: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when header parsing has completed successfully.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_chunked()`](../http1/struct.Parser.html#method.parse_chunked)
    ///
    /// If trailers are supplied.
    ///
    /// [`Parser::parse_headers()`](../http1/struct.Parser.html#method.parse_headers)
    ///
    /// For standard HTTP headers.
    ///
    /// [`Parser::parse_multipart()`](../http1/struct.Parser.html#method.parse_multipart)
    ///
    /// For headers before each multipart section.
    fn on_headers_finished(&mut self) -> bool {
        true
    }

    /// Callback that is executed when a request method has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_headers()`](../http1/struct.Parser.html#method.parse_headers)
    ///
    /// During the initial request line.
    fn on_method(&mut self, method: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when multipart data has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_multipart()`](../http1/struct.Parser.html#method.parse_multipart)
    fn on_multipart_data(&mut self, data: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a response status has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_headers()`](../http1/struct.Parser.html#method.parse_headers)
    ///
    /// During the initial response line.
    fn on_status(&mut self, status: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a response status code has been located.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_headers()`](../http1/struct.Parser.html#method.parse_headers)
    ///
    /// During the initial response line.
    fn on_status_code(&mut self, code: u16) -> bool {
        true
    }

    /// Callback that is executed when a request URL/path has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_headers()`](../http1/struct.Parser.html#method.parse_headers)
    ///
    /// During the initial request line.
    fn on_url(&mut self, url: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a URL encoded field or query string field has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_url_encoded()`](../http1/struct.Parser.html#method.parse_url_encoded)
    fn on_url_encoded_field(&mut self, field: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a URL encoded value or query string value has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_url_encoded()`](../http1/struct.Parser.html#method.parse_url_encoded)
    fn on_url_encoded_value(&mut self, value: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when the HTTP major version has been located.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_headers()`](../http1/struct.Parser.html#method.parse_url_encoded)
    ///
    /// During the initial request or response line.
    fn on_version(&mut self, major: u16, minor: u16) -> bool {
        true
    }
}

// -------------------------------------------------------------------------------------------------

/// Parser context data.
struct ParserContext<'a, T: Http1Handler + 'a> {
    // Current byte.
    byte: u8,

    // Callback handler.
    handler: &'a mut T,

    // Callback mark index.
    mark_index: usize,

    // Stream data.
    stream: &'a [u8],

    // Stream index.
    stream_index: usize
}

impl<'a, T: Http1Handler + 'a> ParserContext<'a, T> {
    /// Create a new `ParserContext`.
    pub fn new(handler: &'a mut T, stream: &'a [u8])
    -> ParserContext<'a, T> {
        ParserContext{ byte:         0,
                       handler:      handler,
                       mark_index:   0,
                       stream:       stream,
                       stream_index: 0 }
    }
}

// -------------------------------------------------------------------------------------------------

/// HTTP 1.1 parser.
pub struct Parser<'a, T: Http1Handler> {
    /// Bit data that stores parser state details, along with HTTP major/minor versions.
    bit_data: u32,

    /// Total byte count processed for headers, and body.
    /// Once the headers are finished processing, this is reset to 0 to track the body length.
    byte_count: usize,

    /// Length storage for max headers length and chunk length.
    length: usize,

    /// Current state.
    state: ParserState,

    /// Current state function.
    state_function: StateFunction<'a, T>
}

impl<'a, T: Http1Handler> Parser<'a, T> {
    /// Create a new `Parser`.
    ///
    /// The initial state `Parser` is set to is a type detection state that determines if the
    /// stream is a HTTP request or HTTP response.
    pub fn new() -> Parser<'a, T> {
        Parser{ bit_data:       0,
                byte_count:     0,
                length:         0,
                state:          ParserState::StripDetect,
                state_function: Parser::strip_detect }
    }

    /// Retrieve the total byte count processed since the instantiation of `Parser`.
    ///
    /// The byte count is updated when any of the parsing functions completes. This means that if a
    /// call to `get_byte_count()` is executed from within a callback, it will be accurate within
    /// `stream.len()` bytes. For precise accuracy, the best time to retrieve the byte count is
    /// outside of all callbacks, and outside of the following functions:
    ///
    /// - `parse_chunked()`
    /// - `parse_headers()`
    /// - `parse_multipart()`
    /// - `parse_url_encoded()`
    pub fn get_byte_count(&self) -> usize {
        self.byte_count
    }

    /// Retrieve the current state.
    pub fn get_state(&self) -> ParserState {
        self.state
    }

    /// Main parser loop.
    #[inline]
    fn parse(&mut self, handler: &mut T, stream: &[u8]) -> Result<Success, ParserError> {
        let mut context = ParserContext::new(handler, stream);

        loop {
            match (self.state_function)(self, &mut context) {
                Ok(ParserValue::Continue) => {
                },
                Ok(ParserValue::Exit(success)) => {
                    self.byte_count += context.stream_index;

                    if let Success::Finished(_) = success {
                        self.state = ParserState::Finished;
                    }

                    return Ok(success);
                },
                Err(error) => {
                    self.byte_count     += context.stream_index;
                    self.state           = ParserState::Dead;
                    self.state_function  = Parser::dead;

                    return Err(error);
                }
            }
        }
    }

    /// Parse chunked transfer encoded data.
    ///
    /// # Arguments
    ///
    /// **`handler`**
    ///
    /// An implementation of [`Http1Handler`](trait.Http1Handler.html) that provides zero or more of
    /// the callbacks used by this function.
    ///
    /// **`stream`**
    ///
    /// The stream of data to be parsed.
    ///
    /// # Callbacks
    ///
    /// - [`Http1Handler::on_body_finished()`](trait.Http1Handler.html#method.on_body_finished)
    /// - [`Http1Handler::on_chunk_data()`](trait.Http1Handler.html#method.on_chunk_data)
    /// - [`Http1Handler::on_chunk_extension_name()`](trait.Http1Handler.html#method.on_chunk_extension_name)
    /// - [`Http1Handler::on_chunk_extension_value()`](trait.Http1Handler.html#method.on_chunk_extension_value)
    /// - [`Http1Handler::on_chunk_length()`](trait.Http1Handler.html#method.on_chunk_length)
    /// - [`Http1Handler::on_header_field()`](trait.Http1Handler.html#method.on_header_field)
    /// - [`Http1Handler::on_header_value()`](trait.Http1Handler.html#method.on_header_value)
    /// - [`Http1Handler::on_headers_finished()`](trait.Http1Handler.html#method.on_headers_finished)
    ///
    /// # Errors
    ///
    /// - [`ParserError::ChunkExtensionName`](enum.ParserError.html#variant.ChunkExtensionName)
    /// - [`ParserError::ChunkExtensionValue`](enum.ParserError.html#variant.ChunkExtensionValue)
    /// - [`ParserError::ChunkLength`](enum.ParserError.html#variant.ChunkLength)
    /// - [`ParserError::CrlfSequence`](enum.ParserError.html#variant.CrlfSequence)
    /// - [`ParserError::MaxChunkLength`](enum.ParserError.html#variant.MaxChunkLength)
    #[inline]
    pub fn parse_chunked(&mut self, handler: &mut T, stream: &[u8])
    -> Result<Success, ParserError> {
        if self.state == ParserState::StripDetect {
            set_flag!(self, F_CHUNKED);

            self.length         = 0;
            self.state          = ParserState::ChunkLength1;
            self.state_function = Parser::chunk_length1;
        }

        self.parse(handler, stream)
    }

    /// Parse initial request/response line and all headers.
    ///
    /// # Arguments
    ///
    /// **`handler`**
    ///
    /// An implementation of [`Http1Handler`](trait.Http1Handler.html) that provides zero or more of
    /// the callbacks used by this function.
    ///
    /// **`stream`**
    ///
    /// The stream of data to be parsed.
    ///
    /// **`max_length`**
    ///
    /// The maximum byte count to process before returning
    /// [`ParserError::MaxHeadersLength`](enum.ParserError.html#variant.MaxHeadersLength).
    ///
    /// Set this to `0` to disable it.
    ///
    /// # Callbacks
    ///
    /// *Request & Response:*
    ///
    /// - [`Http1Handler::on_header_field()`](trait.Http1Handler.html#method.on_header_field)
    /// - [`Http1Handler::on_header_value()`](trait.Http1Handler.html#method.on_header_value)
    /// - [`Http1Handler::on_headers_finished()`](trait.Http1Handler.html#method.on_headers_finished)
    ///
    /// *Request:*
    ///
    /// - [`Http1Handler::on_method()`](trait.Http1Handler.html#method.on_method)
    /// - [`Http1Handler::on_url()`](trait.Http1Handler.html#method.on_url)
    /// - [`Http1Handler::on_version()`](trait.Http1Handler.html#method.on_version)
    ///
    /// *Response:*
    ///
    /// - [`Http1Handler::on_status()`](trait.Http1Handler.html#method.on_status)
    /// - [`Http1Handler::on_status_code()`](trait.Http1Handler.html#method.on_status_code)
    /// - [`Http1Handler::on_version()`](trait.Http1Handler.html#method.on_version)
    ///
    /// # Errors
    ///
    /// - [`ParserError::CrlfSequence`](enum.ParserError.html#variant.CrlfSequence)
    /// - [`ParserError::HeaderField`](enum.ParserError.html#variant.HeaderField)
    /// - [`ParserError::HeaderValue`](enum.ParserError.html#variant.HeaderValue)
    /// - [`ParserError::MaxHeadersLength`](enum.ParserError.html#variant.MaxHeadersLength)
    /// - [`ParserError::Method`](enum.ParserError.html#variant.Method)
    /// - [`ParserError::Status`](enum.ParserError.html#variant.Status)
    /// - [`ParserError::StatusCode`](enum.ParserError.html#variant.StatusCode)
    /// - [`ParserError::Url`](enum.ParserError.html#variant.Url)
    /// - [`ParserError::Version`](enum.ParserError.html#variant.Version)
    #[inline]
    pub fn parse_headers(&mut self, handler: &mut T, mut stream: &[u8], max_length: usize)
    -> Result<Success, ParserError> {
        if max_length == 0 {
            return self.parse(handler, stream);
        }

        if self.state == ParserState::StripDetect {
            self.length = max_length;
        }

        if self.length < stream.len() {
            // amount of data to process is less than the stream length, so let's cut it
            // off and only process what we need
            stream = &stream[0..self.length];
        }

        match self.parse(handler, stream) {
            Ok(Success::Eos(length)) => {
                self.length -= length;

                if self.length > 0 || has_flag!(self, F_HEADERS_FINISHED) {
                    Ok(Success::Eos(length))
                } else {
                    // maximum headers length has been met
                    Err(ParserError::MaxHeadersLength)
                }
            },
            Ok(Success::Finished(length)) => {
                self.length -= length;

                if self.length > 0 || has_flag!(self, F_HEADERS_FINISHED) {
                    Ok(Success::Finished(length))
                } else {
                    // maximum headers length has been met
                    Err(ParserError::MaxHeadersLength)
                }
            },
            Ok(Success::Callback(length)) => {
                self.length -= length;

                if self.length > 0 || has_flag!(self, F_HEADERS_FINISHED) {
                    Ok(Success::Callback(length))
                } else {
                    // maximum headers length has been met
                    Err(ParserError::MaxHeadersLength)
                }
            },
            error => {
                error
            }
        }
    }

    /// Parse multipart data.
    ///
    /// # Arguments
    ///
    /// **`handler`**
    ///
    /// An implementation of [`Http1Handler`](trait.Http1Handler.html) that provides zero or more of
    /// the callbacks used by this function.
    ///
    /// **`stream`**
    ///
    /// The stream of data to be parsed.
    ///
    /// # Callbacks
    ///
    /// - [`Http1Handler::get_boundary()`](trait.Http1Handler.html#method.get_boundary)
    /// - [`Http1Handler::on_body_finished()`](trait.Http1Handler.html#method.on_body_finished)
    /// - [`Http1Handler::on_header_field()`](trait.Http1Handler.html#method.on_header_field)
    /// - [`Http1Handler::on_header_value()`](trait.Http1Handler.html#method.on_header_value)
    /// - [`Http1Handler::on_headers_finished()`](trait.Http1Handler.html#method.on_headers_finished)
    /// - [`Http1Handler::on_multipart_data()`](trait.Http1Handler.html#method.on_multipart_data)
    ///
    /// # Errors
    ///
    /// - [`ParserError::CrlfSequence`](enum.ParserError.html#variant.CrlfSequence)
    /// - [`ParserError::HeaderField`](enum.ParserError.html#variant.HeaderField)
    /// - [`ParserError::HeaderValue`](enum.ParserError.html#variant.HeaderValue)
    #[inline]
    pub fn parse_multipart(&mut self, handler: &mut T, stream: &[u8])
    -> Result<Success, ParserError> {
        if self.state == ParserState::StripDetect {
            self.state          = ParserState::MultipartHyphen1;
            self.state_function = Parser::multipart_hyphen1;
        }

        self.parse(handler, stream)
    }

    /// Parse URL encoded data or query string data.
    ///
    /// # Arguments
    ///
    /// **`handler`**
    ///
    /// An implementation of [`Http1Handler`](trait.Http1Handler.html) that provides zero or more of
    /// the callbacks used by this function.
    ///
    /// **`stream`**
    ///
    /// The stream of data to be parsed.
    ///
    /// **`length`**
    ///
    /// # Callbacks
    ///
    /// - [`Http1Handler::on_body_finished()`](trait.Http1Handler.html#method.on_body_finished)
    /// - [`Http1Handler::on_url_encoded_field()`](trait.Http1Handler.html#method.on_url_encoded_field)
    /// - [`Http1Handler::on_url_encoded_value()`](trait.Http1Handler.html#method.on_url_encoded_value)
    ///
    /// # Errors
    ///
    /// - [`ParserError::UrlEncodedField`](enum.ParserError.html#variant.UrlEncodedField)
    /// - [`ParserError::UrlEncodedValue`](enum.ParserError.html#variant.UrlEncodedValue)
    #[inline]
    pub fn parse_url_encoded(&mut self, handler: &mut T, mut stream: &[u8], length: usize)
    -> Result<Success, ParserError> {
        if self.state == ParserState::StripDetect {
            self.length         = length;
            self.state          = ParserState::UrlEncodedField;
            self.state_function = Parser::url_encoded_field;
        } else if self.state == ParserState::Finished {
            // already finished
            return Ok(Success::Finished(0));
        }

        if self.length < stream.len() {
            // amount of data to process is less than the stream length, so let's trim
            // the stream to match the proper length (we won't process the rest anyways)
            stream = &stream[0..self.length];
        }

        match self.parse(handler, stream) {
            Ok(Success::Eos(length)) => {
                if self.length - length == 0 {
                    self.state          = ParserState::Finished;
                    self.state_function = Parser::finished;

                    if handler.on_body_finished() {
                        Ok(Success::Finished(stream.len()))
                    } else {
                        Ok(Success::Callback(stream.len()))
                    }
                } else {
                    self.length -= length;

                    Ok(Success::Eos(length))
                }
            },
            Ok(Success::Callback(length)) => {
                self.length -= length;

                Ok(Success::Callback(length))
            },
            other => {
                other
            }
        }
    }

    /// Reset the parser back to its initial state.
    pub fn reset(&mut self) {
        self.bit_data       = 0;
        self.byte_count     = 0;
        self.length         = 0;
        self.state          = ParserState::Detect1;
        self.state_function = Parser::detect1;
    }

    // ---------------------------------------------------------------------------------------------
    // DETECTION STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn strip_detect(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(context,
            // on end-of-stream
            exit_eos!(self, context)
        );

        transition_fast!(self, context, ParserState::Detect1, detect1);
    }

    #[inline]
    fn detect1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        macro_rules! version {
            ($major:expr, $minor:expr, $length:expr) => ({
                bs_jump!(context, $length);
                set_state!(self, ParserState::StripResponseStatusCode, strip_response_status_code);

                if context.handler.on_version($major, $minor) {
                    transition_fast!(self, context);
                } else {
                    exit_callback!(self, context);
                }
            });
        }

        if bs_starts_with1!(context, b"H") || bs_starts_with1!(context, b"h") {
            if bs_has_bytes!(context, 9) {
                if bs_starts_with9!(context, b"HTTP/1.1 ") {
                    version!(1, 1, 9);
                } else if bs_starts_with9!(context, b"HTTP/2.0 ") {
                    version!(2, 0, 9);
                } else if bs_starts_with9!(context, b"HTTP/1.0 ") {
                    version!(1, 0, 9);
                } else if bs_starts_with5!(context, b"HTTP/") {
                    bs_jump!(context, 5);

                    transition_fast!(self, context,
                                     ParserState::ResponseVersionMajor, response_version_major);
                }
            } else {
                bs_jump!(context, 1);

                transition_fast!(self, context, ParserState::Detect2, detect2);
            }
        }

        // this is a request
        transition_fast!(self, context, ParserState::RequestMethod, request_method);
    }

    #[inline]
    fn detect2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'T' || context.byte == b't' {
            transition_fast!(self, context, ParserState::Detect3, detect3);
        }

        // since we're in a detection state and didn't know until right here that we're moving from
        // detection -> request, we need to manually submit the first n bytes of the of the request
        // method, and the request method state will do the rest of the work for us
        bs_replay!(context);

        callback_transition_fast!(self, context, on_method, b"H",
                                  ParserState::RequestMethod, request_method);
    }

    #[inline]
    fn detect3(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'T' || context.byte == b't' {
            transition_fast!(self, context, ParserState::Detect4, detect4);
        }

        // since we're in a detection state and didn't know until right here that we're moving from
        // detection -> request, we need to manually submit the first n bytes of the of the request
        // method, and the request method state will do the rest of the work for us
        bs_replay!(context);

        callback_transition_fast!(self, context, on_method, b"HT",
                                  ParserState::RequestMethod, request_method);
    }

    #[inline]
    fn detect4(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'P' || context.byte == b'p' {
            transition_fast!(self, context, ParserState::Detect5, detect5);
        }

        // since we're in a detection state and didn't know until right here that we're moving from
        // detection -> request, we need to manually submit the first n bytes of the of the request
        // method, and the request method state will do the rest of the work for us
        bs_replay!(context);

        callback_transition_fast!(self, context, on_method, b"HTT",
                                  ParserState::RequestMethod, request_method);
    }

    #[inline]
    fn detect5(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'/' {
            set_lower14!(self, 0);
            set_upper14!(self, 0);

            transition_fast!(self, context, ParserState::ResponseVersionMajor, response_version_major);
        }

        // since we're in a detection state and didn't know until right here that we're moving from
        // detection -> request, we need to manually submit the first n bytes of the of the request
        // method, and the request method state will do the rest of the work for us
        bs_replay!(context);

        callback_transition_fast!(self, context, on_method, b"HTTP",
                                  ParserState::RequestMethod, request_method);
    }

    // ---------------------------------------------------------------------------------------------
    // HEADER STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn pre_headers1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition_fast!(self, context, ParserState::PreHeaders2, pre_headers2);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn pre_headers2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, ParserState::Newline4, newline4);
        } else {
            bs_replay!(context);

            transition_fast!(self, context, ParserState::StripHeaderField, strip_header_field);
        }
    }

    #[inline]
    fn strip_header_field(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(context,
            // on end-of-stream
            exit_eos!(self, context)
        );

        transition_fast!(self, context, ParserState::FirstHeaderField, first_header_field);
    }

    #[inline]
    #[cfg_attr(test, allow(cyclomatic_complexity))]
    fn first_header_field(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        macro_rules! field {
            ($header:expr, $length:expr) => ({
                bs_jump!(context, $length);

                callback_transition_fast!(self, context,
                                          on_header_field, $header,
                                          ParserState::StripHeaderValue, strip_header_value);
            });
        }

        if bs_has_bytes!(context, 24) {
            // have enough bytes to compare common header fields immediately, without collecting
            // individual tokens
            if context.byte == b'C' {
                if bs_starts_with11!(context, b"Connection:") {
                    field!(b"connection", 11);
                } else if bs_starts_with13!(context, b"Content-Type:") {
                    field!(b"content-type", 13);
                } else if bs_starts_with15!(context, b"Content-Length:") {
                    field!(b"content-length", 15);
                } else if bs_starts_with7!(context, b"Cookie:") {
                    field!(b"cookie", 7);
                } else if bs_starts_with14!(context, b"Cache-Control:") {
                    field!(b"cache-control", 14);
                } else if bs_starts_with24!(context, b"Content-Security-Policy:") {
                    field!(b"content-security-policy", 24);
                }
            } else if context.byte == b'A' {
                if bs_starts_with7!(context, b"Accept:") {
                    field!(b"accept", 7);
                } else if bs_starts_with15!(context, b"Accept-Charset:") {
                    field!(b"accept-charset", 15);
                } else if bs_starts_with16!(context, b"Accept-Encoding:") {
                    field!(b"accept-encoding", 16);
                } else if bs_starts_with16!(context, b"Accept-Language:") {
                    field!(b"accept-language", 16);
                } else if bs_starts_with14!(context, b"Authorization:") {
                    field!(b"authorization", 14);
                }
            } else if context.byte == b'L' {
                if bs_starts_with9!(context, b"Location:") {
                    field!(b"location", 9);
                } else if bs_starts_with14!(context, b"Last-Modified:") {
                    field!(b"last-modified", 14);
                }
            } else if bs_starts_with7!(context, b"Pragma:") {
                field!(b"pragma", 7);
            } else if bs_starts_with11!(context, b"Set-Cookie:") {
                field!(b"set-cookie", 11);
            } else if bs_starts_with18!(context, b"Transfer-Encoding:") {
                field!(b"transfer-encoding", 18);
            } else if context.byte == b'U' {
                if bs_starts_with11!(context, b"User-Agent:") {
                    field!(b"user-agent", 11);
                } else if bs_starts_with8!(context, b"Upgrade:") {
                    field!(b"upgrade", 8);
                }
            } else if context.byte == b'X' {
                if bs_starts_with13!(context, b"X-Powered-By:") {
                    field!(b"x-powered-by", 13);
                } else if bs_starts_with16!(context, b"X-Forwarded-For:") {
                    field!(b"x-forwarded-for", 16);
                } else if bs_starts_with17!(context, b"X-Forwarded-Host:") {
                    field!(b"x-forwarded-host", 17);
                } else if bs_starts_with17!(context, b"X-XSS-Protection:") {
                    field!(b"x-xss-protection", 17);
                } else if bs_starts_with13!(context, b"X-WebKit-CSP:") {
                    field!(b"x-webkit-csp", 13);
                }
            } else if bs_starts_with17!(context, b"WWW-Authenticate:") {
                field!(b"www-authenticate", 17);
            }
        }

        transition_fast!(self, context, ParserState::UpperHeaderField, upper_header_field);
    }

    #[inline]
    fn upper_header_field(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte > 0x40 && context.byte < 0x5B {
            // upper-cased byte, let's lower-case it
            callback_transition!(self, context,
                                 on_header_field, &[context.byte + 0x20],
                                 ParserState::LowerHeaderField, lower_header_field);
        }

        bs_replay!(context);

        transition!(self, context,
                    ParserState::LowerHeaderField, lower_header_field);
    }

    #[inline]
    fn lower_header_field(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_tokens!(context, ParserError::HeaderField,
            // stop on these bytes
               context.byte == b':'
            || (context.byte > 0x40 && context.byte < 0x5B),

            // on end-of-stream
            callback_eos_expr!(self, context, on_header_field)
        );

        if context.byte == b':' {
            callback_ignore_transition_fast!(self, context,
                                             on_header_field,
                                             ParserState::StripHeaderValue, strip_header_value);
        }

        // upper-cased byte
        bs_replay!(context);

        callback_transition_fast!(self, context,
                                  on_header_field,
                                  ParserState::UpperHeaderField, upper_header_field);
    }

    #[inline]
    fn strip_header_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(context,
            // on end-of-stream
            exit_eos!(self, context)
        );

        bs_next!(context);

        if context.byte == b'"' {
            transition_fast!(self, context, ParserState::HeaderQuotedValue, header_quoted_value);
        }

        bs_replay!(context);

        transition_fast!(self, context, ParserState::HeaderValue, header_value);
    }

    #[inline]
    fn header_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_field!(context, ParserError::HeaderValue, b'\r',
            callback_eos_expr!(self, context, on_header_value)
        );

        callback_ignore_transition_fast!(self, context,
                                         on_header_value,
                                         ParserState::Newline2, newline2);
    }

    #[inline]
    fn header_quoted_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_quoted_field!(context, ParserError::HeaderValue,
            // on end-of-stream
            {
                callback_eos_expr!(self, context, on_header_value);
            }
        );

        if context.byte == b'"' {
            callback_ignore_transition_fast!(self, context,
                                             on_header_value,
                                             ParserState::Newline1, newline1);
        } else {
            callback_ignore_transition_fast!(self, context,
                                             on_header_value,
                                             ParserState::HeaderEscapedValue, header_escaped_value);
        }
    }

    #[inline]
    fn header_escaped_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        callback_transition!(self, context,
                             on_header_value, &[context.byte],
                             ParserState::HeaderQuotedValue, header_quoted_value);
    }

    #[inline]
    fn newline1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        if bs_has_bytes!(context, 2) && bs_starts_with2!(context, b"\r\n") {
            bs_jump!(context, 2);

            transition_fast!(self, context, ParserState::Newline3, newline3);
        }

        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, ParserState::Newline2, newline2);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn newline2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition_fast!(self, context, ParserState::Newline3, newline3);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn newline3(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, ParserState::Newline4, newline4);
        } else if context.byte == b' ' || context.byte == b'\t' {
            // multiline header value
            callback_transition!(self, context,
                                 on_header_value, b" ",
                                 ParserState::StripHeaderValue, strip_header_value);
        } else {
            bs_replay!(context);
            transition!(self, context, ParserState::StripHeaderField, strip_header_field);
        }
    }

    #[inline]
    fn newline4(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            set_flag!(self, F_HEADERS_FINISHED);

            if has_flag!(self, F_CHUNKED) {
                set_state!(self, ParserState::BodyFinished, body_finished);
            } else {
                set_state!(self, ParserState::Finished, finished);
            }

            if context.handler.on_headers_finished() {
                transition_fast!(self, context);
            } else {
                exit_callback!(self, context);
            }
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    // ---------------------------------------------------------------------------------------------
    // REQUEST STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn request_method(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        macro_rules! method {
            ($method:expr, $length:expr) => (
                bs_jump!(context, $length);

                callback_transition_fast!(self, context,
                                          on_method, $method,
                                          ParserState::StripRequestUrl, strip_request_url);
            );
        }

        if bs_has_bytes!(context, 8) {
            // have enough bytes to compare all known methods immediately, without collecting
            // individual tokens

            // get the first byte, then replay it (for use with bs_starts_with!())
            bs_next!(context);
            bs_replay!(context);

            if bs_starts_with4!(context, b"GET ") {
                method!(b"GET", 4);
            } else if context.byte == b'P' {
                if bs_starts_with5!(context, b"POST ") {
                    method!(b"POST", 5);
                } else if bs_starts_with4!(context, b"PUT ") {
                    method!(b"PUT", 4);
                }
            } else if bs_starts_with7!(context, b"DELETE ") {
                method!(b"DELETE", 7);
            } else if bs_starts_with8!(context, b"CONNECT ") {
                method!(b"CONNECT", 8);
            } else if bs_starts_with8!(context, b"OPTIONS ") {
                method!(b"OPTIONS", 8);
            } else if bs_starts_with5!(context, b"HEAD ") {
                method!(b"HEAD", 5);
            } else if bs_starts_with6!(context, b"TRACE ") {
                method!(b"TRACE", 6);
            }
        }

        collect_tokens!(context, ParserError::Method,
            // stop on these bytes
            context.byte == b' ',

            // on end-of-stream
            callback_eos_expr!(self, context, on_method)
        );

        bs_replay!(context);

        callback_transition_fast!(self, context,
                                  on_method,
                                  ParserState::StripRequestUrl, strip_request_url);
    }

    #[inline]
    fn strip_request_url(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(context,
            // on end-of-stream
            exit_eos!(self, context)
        );

        transition_fast!(self, context, ParserState::RequestUrl, request_url);
    }

    #[inline]
    fn request_url(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_visible!(context, ParserError::Url,
            // stop on these bytes
            context.byte == b' ',

            // on end-of-stream
            callback_eos_expr!(self, context, on_url)
        );

        bs_replay!(context);

        callback_transition_fast!(self, context,
                                  on_url,
                                  ParserState::StripRequestHttp, strip_request_http);
    }

    #[inline]
    fn strip_request_http(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(context,
            // on end-of-stream
            exit_eos!(self, context)
        );

        transition_fast!(self, context, ParserState::RequestHttp1, request_http1);
    }

    #[inline]
    fn request_http1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        macro_rules! version {
            ($major:expr, $minor:expr, $length:expr) => (
                bs_jump!(context, $length);
                set_state!(self, ParserState::PreHeaders1, pre_headers1);

                if context.handler.on_version($major, $minor) {
                    transition_fast!(self, context);
                } else {
                    exit_callback!(self, context);
                }
            );
        }

        if bs_has_bytes!(context, 9) {
            // have enough bytes to compare all known versions immediately, without collecting
            // individual tokens
            if bs_starts_with9!(context, b"HTTP/1.1\r") {
                version!(1, 1, 9);
            } else if bs_starts_with9!(context, b"HTTP/2.0\r") {
                version!(2, 0, 9);
            } else if bs_starts_with9!(context, b"HTTP/1.0\r") {
                version!(1, 0, 9);
            }
        }

        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'H' || context.byte == b'h' {
            transition_fast!(self, context, ParserState::RequestHttp2, request_http2);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_http2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'T' || context.byte == b't' {
            transition_fast!(self, context, ParserState::RequestHttp3, request_http3);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_http3(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'T' || context.byte == b't' {
            transition_fast!(self, context, ParserState::RequestHttp4, request_http4);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_http4(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'P' || context.byte == b'p' {
            transition_fast!(self, context, ParserState::RequestHttp5, request_http5);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_http5(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'/' {
            set_lower14!(self, 0);
            set_upper14!(self, 0);

            transition_fast!(self, context, ParserState::RequestVersionMajor, request_version_major);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_version_major(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = get_lower14!(self);

        collect_digits32!(context, ParserError::Version, digit, 999, {
            set_lower14!(self, digit);

            exit_eos!(self, context);
        });

        set_lower14!(self, digit);

        if context.byte == b'.' {
            transition_fast!(self, context, ParserState::RequestVersionMinor, request_version_minor);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_version_minor(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = get_upper14!(self);

        collect_digits32!(context, ParserError::Version, digit, 999, {
            set_upper14!(self, digit);

            exit_eos!(self, context);
        });

        set_state!(self, ParserState::PreHeaders1, pre_headers1);

        if context.handler.on_version(get_lower14!(self) as u16, digit as u16) {
            transition_fast!(self, context);
        } else {
            exit_callback!(self, context);
        }
    }

    // ---------------------------------------------------------------------------------------------
    // RESPONSE STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn response_version_major(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = get_lower14!(self);

        collect_digits32!(context, ParserError::Version, digit, 999, {
            set_lower14!(self, digit);

            exit_eos!(self, context);
        });

        set_lower14!(self, digit);

        if context.byte == b'.' {
            transition_fast!(self, context, ParserState::ResponseVersionMinor, response_version_minor);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn response_version_minor(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = get_upper14!(self);

        collect_digits32!(context, ParserError::Version, digit, 999, {
            set_upper14!(self, digit);

            exit_eos!(self, context);
        });

        set_state!(self, ParserState::StripResponseStatusCode, strip_response_status_code);

        if context.handler.on_version(get_lower14!(self) as u16, digit as u16) {
            transition_fast!(self, context);
        } else {
            exit_callback!(self, context);
        }
    }

    #[inline]
    fn strip_response_status_code(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(context,
            // on end-of-stream
            exit_eos!(self, context)
        );

        bs_next!(context);

        if !is_digit!(context.byte) {
            return Err(ParserError::StatusCode(context.byte));
        }

        bs_replay!(context);

        set_lower14!(self, 0);

        transition_fast!(self, context, ParserState::ResponseStatusCode, response_status_code);
    }

    #[inline]
    fn response_status_code(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = get_lower14!(self);

        collect_digits32!(context, ParserError::StatusCode, digit, 999, {
            set_lower14!(self, digit);
            exit_eos!(self, context);
        });

        bs_replay!(context);
        set_state!(self, ParserState::StripResponseStatus, strip_response_status);

        if context.handler.on_status_code(digit as u16) {
            transition_fast!(self, context);
        } else {
            exit_callback!(self, context);
        }
    }

    #[inline]
    fn strip_response_status(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(context,
            // on end-of-stream
            exit_eos!(self, context)
        );

        transition_fast!(self, context, ParserState::ResponseStatus, response_status);
    }

    #[inline]
    fn response_status(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        bs_collect!(context,
            // collect loop
            if context.byte == b'\r' {
                break;
            } else if is_token(context.byte) || context.byte == b' ' || context.byte == b'\t' {
                // do nothing
            } else {
                return Err(ParserError::Status(context.byte));
            },

            // on end-of-stream
            callback!(self, context, on_status, {
                exit_eos!(self, context);
            })
        );

        callback_ignore_transition_fast!(self, context,
                                         on_status,
                                         ParserState::PreHeaders1, pre_headers1);
    }

    // ---------------------------------------------------------------------------------------------
    // BODY STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn chunk_length1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'0' {
            transition_fast!(self, context, ParserState::ChunkLengthEnd, chunk_length_end);
        } else if !is_hex!(context.byte) {
            return Err(ParserError::ChunkLength(context.byte));
        }

        bs_replay!(context);

        transition_fast!(self, context, ParserState::ChunkLength2, chunk_length2);
    }

    #[inline]
    fn chunk_length2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        match hex_to_byte(&[context.byte]) {
            Some(byte) => {
                if (self.length << 4) + byte as usize > 0xFFFFFFF {
                    // limit chunk length to 28 bits
                    return Err(ParserError::MaxChunkLength);
                }

                self.length <<= 4;
                self.length  += byte as usize;

                transition!(self, context, ParserState::ChunkLength2, chunk_length2);
            },
            None => {
                bs_replay!(context);

                transition_fast!(self, context, ParserState::ChunkLengthEnd, chunk_length_end);
            }
        }
    }

    #[inline]
    fn chunk_length_end(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            if self.length == 0 {
                callback_transition_fast!(self, context,
                                          on_chunk_length, self.length,
                                          ParserState::Newline2, newline2);
            }

            callback_transition_fast!(self, context,
                                      on_chunk_length, self.length,
                                      ParserState::ChunkLengthNewline, chunk_length_newline);
        } else if context.byte == b';' {
            set_flag!(self, F_CHUNK_EXTENSIONS);

            callback_transition_fast!(self, context,
                                      on_chunk_length, self.length,
                                      ParserState::StripChunkExtensionName,
                                      strip_chunk_extension_name);
        }

        Err(ParserError::ChunkLength(context.byte))
    }

    #[inline]
    fn strip_chunk_extension_name(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(context,
            // on end-of-stream
            exit_eos!(self, context)
        );

        transition_fast!(self, context,
                         ParserState::UpperChunkExtensionName, upper_chunk_extension_name);
    }

    #[inline]
    fn upper_chunk_extension_name(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte > 0x40 && context.byte < 0x5B {
            callback_transition!(self, context,
                                 on_chunk_extension_name, &[context.byte + 0x20],
                                 ParserState::LowerChunkExtensionName, lower_chunk_extension_name);
        }

        bs_replay!(context);

        transition!(self, context,
                    ParserState::LowerChunkExtensionName, lower_chunk_extension_name);
    }

    #[inline]
    fn lower_chunk_extension_name(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_tokens!(context, ParserError::ChunkExtensionName,
            // stop on these bytes
               context.byte == b'='
            || context.byte == b'\r'
            || context.byte == b';'
            || (context.byte > 0x40 && context.byte < 0x5B),

            // on end-of-stream
            callback_eos_expr!(self, context, on_chunk_extension_name)
        );

        if context.byte == b'=' {
            callback_ignore_transition_fast!(self, context,
                                             on_chunk_extension_name,
                                             ParserState::ChunkExtensionValue, chunk_extension_value);
        } else if context.byte == b'\r' || context.byte == b';' {
            // extension name without a value
            bs_replay!(context);

            callback_transition_fast!(self, context,
                                      on_chunk_extension_name,
                                      ParserState::ChunkExtensionNameNoValue,
                                      chunk_extension_name_no_value);
        } else {
            // upper-cased byte
            bs_replay!(context);

            callback_transition_fast!(self, context,
                                      on_chunk_extension_name,
                                      ParserState::UpperChunkExtensionName,
                                      upper_chunk_extension_name);
        }
    }

    #[inline]
    fn chunk_extension_name_no_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            set_state!(self, ParserState::ChunkLengthNewline, chunk_length_newline);
        } else {
            set_state!(self, ParserState::UpperChunkExtensionName, upper_chunk_extension_name);
        }

        // since the chunk extension has no value, let's send an empty one
        if context.handler.on_chunk_extension_value(b"") {
            transition!(self, context);
        } else {
            exit_callback!(self, context);
        }
    }

    #[inline]
    fn strip_chunk_extension_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(context,
            // on end-of-stream
            exit_eos!(self, context)
        );

        transition_fast!(self, context,
                         ParserState::ChunkExtensionValue, chunk_extension_value);
    }

    #[inline]
    fn chunk_extension_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_tokens!(context, ParserError::ChunkExtensionValue,
            // stop on these bytes
               context.byte == b'\r'
            || context.byte == b';'
            || context.byte == b'"',

            // on end-of-stream
            callback_eos_expr!(self, context, on_chunk_extension_value)
        );

        match context.byte {
            b'\r' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_chunk_extension_value,
                                                 ParserState::ChunkLengthNewline, chunk_length_newline);
            },
            b';' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_chunk_extension_value,
                                                 ParserState::UpperChunkExtensionName,
                                                 upper_chunk_extension_name);
            },
            _ => {
                transition_fast!(self, context, ParserState::ChunkExtensionQuotedValue,
                                 chunk_extension_quoted_value);
            }
        }
    }

    #[inline]
    fn chunk_extension_quoted_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_quoted_field!(context, ParserError::ChunkExtensionValue,
            // on end-of-stream
            {
                callback_eos_expr!(self, context, on_chunk_extension_value);
            }
        );

        if context.byte == b'"' {
            callback_ignore_transition_fast!(self, context,
                                             on_chunk_extension_value,
                                             ParserState::ChunkExtensionSemiColon,
                                             chunk_extension_semi_colon);
        }

        callback_ignore_transition_fast!(self, context,
                                         on_chunk_extension_value,
                                         ParserState::ChunkExtensionEscapedValue,
                                         chunk_extension_escaped_value);
    }

    #[inline]
    fn chunk_extension_escaped_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_visible_7bit!(context.byte) || context.byte == b' ' {
            callback_transition_fast!(self, context,
                                      on_chunk_extension_value, &[context.byte],
                                      ParserState::ChunkExtensionQuotedValue,
                                      chunk_extension_quoted_value);
        }

        Err(ParserError::ChunkExtensionValue(context.byte))
    }

    #[inline]
    fn chunk_extension_semi_colon(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b';' {
            transition!(self, context,
                        ParserState::UpperChunkExtensionName, upper_chunk_extension_name);
        } else if context.byte == b'\r' {
            transition!(self, context, ParserState::ChunkLengthNewline, chunk_length_newline);
        }

        Err(ParserError::ChunkExtensionName(context.byte))
    }

    #[inline]
    fn chunk_length_newline(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            set_state!(self, ParserState::ChunkData, chunk_data);

            if has_flag!(self, F_CHUNK_EXTENSIONS) {
                if context.handler.on_chunk_extensions_finished() {
                    transition!(self, context);
                } else {
                    exit_callback!(self, context);
                }
            } else {
                transition!(self, context);
            }
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn chunk_data(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= self.length {
            bs_collect_length!(context, self.length);

            self.length = 0;

            callback_transition!(self, context,
                                 on_chunk_data,
                                 ParserState::ChunkDataNewline1, chunk_data_newline1);
        }

        self.length -= bs_available!(context);

        bs_collect_length!(context, bs_available!(context));

        callback_transition!(self, context,
                             on_chunk_data,
                             ParserState::ChunkData, chunk_data);
    }

    #[inline]
    fn chunk_data_newline1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, ParserState::ChunkDataNewline2, chunk_data_newline2);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn chunk_data_newline2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition_fast!(self, context, ParserState::ChunkLength1, chunk_length1);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn multipart_hyphen1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'-' {
            transition_fast!(self, context, ParserState::MultipartHyphen2, multipart_hyphen2);
        }

        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    fn multipart_hyphen2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'-' {
            transition_fast!(self, context, ParserState::MultipartTryBoundary, multipart_try_boundary);
        }

        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    fn multipart_try_hyphen1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn multipart_try_hyphen2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn multipart_try_boundary(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn multipart_newline1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn multipart_newline2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn multipart_data(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn url_encoded_field(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_visible!(context, ParserError::UrlEncodedField,
            // stop on these bytes
               context.byte == b'='
            || context.byte == b'%'
            || context.byte == b'&'
            || context.byte == b'+',

            // on end-of-stream
            callback_eos_expr!(self, context, on_url_encoded_field)
        );

        match context.byte {
            b'=' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_field,
                                                 ParserState::UrlEncodedValue, url_encoded_value);
            },
            b'%' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_field,
                                                 ParserState::UrlEncodedFieldHex, url_encoded_field_hex);
            },
            b'&' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_field,
                                                 ParserState::UrlEncodedFieldAmpersand,
                                                 url_encoded_field_ampersand);
            },
            _ => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_field,
                                                 ParserState::UrlEncodedFieldPlus,
                                                 url_encoded_field_plus);
            }
        }
    }

    #[inline]
    fn url_encoded_field_ampersand(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        // no value, send an empty one
        callback_transition!(self, context,
                             on_url_encoded_value, b"",
                             ParserState::UrlEncodedField, url_encoded_field);
    }

    #[inline]
    fn url_encoded_field_hex(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        if bs_has_bytes!(context, 2) {
            bs_jump!(context, 2);

            match hex_to_byte(bs_slice!(context)) {
                Some(byte) => {
                    callback_transition!(self, context,
                                         on_url_encoded_field, &[byte],
                                         ParserState::UrlEncodedField, url_encoded_field);
                },
                _ => {
                    return Err(ParserError::UrlEncodedField(context.byte));
                }
            }
        }

        exit_eos!(self, context);
    }

    #[inline]
    fn url_encoded_field_plus(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        callback_transition!(self, context,
                             on_url_encoded_field, b" ",
                             ParserState::UrlEncodedField, url_encoded_field);
    }

    #[inline]
    fn url_encoded_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_visible!(context, ParserError::UrlEncodedValue,
            // stop on these bytes
               context.byte == b'%'
            || context.byte == b'&'
            || context.byte == b'+'
            || context.byte == b'=',

            // on end-of-stream
            callback_eos_expr!(self, context, on_url_encoded_value)
        );

        match context.byte {
            b'%' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_value,
                                                 ParserState::UrlEncodedValueHex, url_encoded_value_hex);
            },
            b'&' => {
                callback_ignore_transition!(self, context,
                                            on_url_encoded_value,
                                            ParserState::UrlEncodedField,
                                            url_encoded_field);
            },
            b'+' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_value,
                                                 ParserState::UrlEncodedValuePlus,
                                                 url_encoded_value_plus);
            },
            _ => {
                Err(ParserError::UrlEncodedValue(context.byte))
            }
        }
    }

    #[inline]
    fn url_encoded_value_hex(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        if bs_has_bytes!(context, 2) {
            bs_jump!(context, 2);

            match hex_to_byte(bs_slice!(context)) {
                Some(byte) => {
                    callback_transition!(self, context,
                                         on_url_encoded_value, &[byte],
                                         ParserState::UrlEncodedValue, url_encoded_value);
                },
                _ => {
                    return Err(ParserError::UrlEncodedValue(context.byte));
                }
            }
        }

        exit_eos!(self, context);
    }

    #[inline]
    fn url_encoded_value_plus(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        callback_transition!(self, context,
                             on_url_encoded_value, b" ",
                             ParserState::UrlEncodedValue, url_encoded_value);
    }

    // ---------------------------------------------------------------------------------------------
    // DEAD & FINISHED STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn dead(&mut self, _context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        Err(ParserError::Dead)
    }

    #[inline]
    fn body_finished(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        set_state!(self, ParserState::Finished, finished);

        if context.handler.on_body_finished() {
            transition_fast!(self, context);
        } else {
            exit_callback!(self, context);
        }
    }

    #[inline]
    fn finished(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_finished!(self, context);
    }
}
