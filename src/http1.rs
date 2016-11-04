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

use byte::is_token;
use fsm::{ ParserValue,
           Success };

use std::{ fmt,
           str };

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

/// Parsing chunk encoded.
const F_CHUNKED: u32 = 1;

/// Parsing chunk encoded extensions.
const F_CHUNK_EXTENSIONS: u32 = 1 << 1;

/// Headers are finished parsing.
const F_HEADERS_FINISHED: u32 = 1 << 2;

/// Parsing multipart.
const F_MULTIPART: u32 = 1 << 3;

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

/// Increase the lower 14 bits.
macro_rules! inc_lower14 {
    ($parser:expr, $length:expr) => ({
        set_lower14!($parser, get_lower14!($parser) as usize + $length as usize);
    });
}

/// Increase the upper 14 bits.
macro_rules! inc_upper14 {
    ($parser:expr, $length:expr) => ({
        set_upper14!($parser, get_upper14!($parser) as usize + $length as usize);
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

/// `DebugHandler` works with all `Parser` parsing methods.
///
/// When in use, all parsed bytes will be printed, along with the callback name and length
/// of parsed data.
///
/// If you're debugging large sets of data, it's a good idea to pass fairly small chunks
/// of stream data at a time, about *4096* bytes or so. And in between parser function calls, if
/// you don't need to retain the data, execute
/// [`reset()`](struct.DebugHandler.html#method.reset) so that vectors
/// collecting the data don't consume too much memory.
#[derive(Default)]
pub struct DebugHandler {
    /// Indicates that the body has successfully been parsed.
    pub body_finished: bool,

    /// Chunk data.
    pub chunk_data: Vec<u8>,

    /// Chunk extension name.
    pub chunk_extension_name:  Vec<u8>,

    /// Chunk extension value.
    pub chunk_extension_value: Vec<u8>,

    /// Chunk length.
    pub chunk_length: usize,

    /// Header field.
    pub header_field: Vec<u8>,

    /// Header value.
    pub header_value: Vec<u8>,

    /// Indicates that headers have successfully been parsed.
    pub headers_finished: bool,

    /// Request method.
    pub method: Vec<u8>,

    /// Multipart data.
    pub multipart_data: Vec<u8>,

    /// Response status.
    pub status: Vec<u8>,

    /// Response status code.
    pub status_code: u16,

    /// Indicates that the status line has successfully been parsed.
    pub status_finished: bool,

    /// Request URL.
    pub url: Vec<u8>,

    /// URL encoded field.
    pub url_encoded_field: Vec<u8>,

    /// URL encoded value.
    pub url_encoded_value: Vec<u8>,

    /// HTTP major version.
    pub version_major: u16,

    /// HTTP minor version.
    pub version_minor: u16
}

impl DebugHandler {
    /// Create a new `DebugHandler`.
    pub fn new() -> DebugHandler {
        DebugHandler{ body_finished:         false,
                      chunk_data:            Vec::new(),
                      chunk_extension_name:  Vec::new(),
                      chunk_extension_value: Vec::new(),
                      chunk_length:          0,
                      header_field:          Vec::new(),
                      header_value:          Vec::new(),
                      headers_finished:      false,
                      method:                Vec::new(),
                      multipart_data:        Vec::new(),
                      status:                Vec::new(),
                      status_code:           0,
                      status_finished:       false,
                      url:                   Vec::new(),
                      url_encoded_field:     Vec::new(),
                      url_encoded_value:     Vec::new(),
                      version_major:         0,
                      version_minor:         0 }
    }

    /// Reset the handler to its original state.
    pub fn reset(&mut self) {
        self.body_finished         = false;
        self.chunk_data            = Vec::new();
        self.chunk_extension_name  = Vec::new();
        self.chunk_extension_value = Vec::new();
        self.chunk_length          = 0;
        self.header_field          = Vec::new();
        self.header_value          = Vec::new();
        self.headers_finished      = false;
        self.method                = Vec::new();
        self.multipart_data        = Vec::new();
        self.status                = Vec::new();
        self.status_code           = 0;
        self.status_finished       = false;
        self.url                   = Vec::new();
        self.url_encoded_field     = Vec::new();
        self.url_encoded_value     = Vec::new();
        self.version_major         = 0;
        self.version_minor         = 0;
    }
}

impl HttpHandler for DebugHandler {
    fn content_length(&mut self) -> Option<usize> {
        None
    }

    fn on_body_finished(&mut self) -> bool {
        println!("on_body_finished");
        true
    }

    fn on_chunk_data(&mut self, data: &[u8]) -> bool {
        self.chunk_data.extend_from_slice(data);

        for byte in data {
            if !is_visible_7bit!(*byte) {
                println!("on_chunk_data [{}]: *hidden*", data.len());
                return true;
            }
        }

        println!("on_chunk_data [{}]: {:?}", data.len(), str::from_utf8(data).unwrap());
        true
    }

    fn on_chunk_extension_finished(&mut self) -> bool {
        true
    }

    fn on_chunk_extension_name(&mut self, name: &[u8]) -> bool {
        println!("on_chunk_extension_name [{}]: {:?}", name.len(), str::from_utf8(name).unwrap());
        self.chunk_extension_name.extend_from_slice(name);
        true
    }

    fn on_chunk_extension_value(&mut self, value: &[u8]) -> bool {
        println!("on_chunk_extension_value [{}]: {:?}", value.len(), str::from_utf8(value).unwrap());
        self.chunk_extension_value.extend_from_slice(value);
        true
    }

    fn on_chunk_length(&mut self, length: usize) -> bool {
        println!("on_chunk_length: {}", length);
        self.chunk_length = length;
        true
    }

    fn on_header_field(&mut self, field: &[u8]) -> bool {
        println!("on_header_field [{}]: {:?}", field.len(), str::from_utf8(field).unwrap());
        self.header_field.extend_from_slice(field);
        true
    }

    fn on_header_value(&mut self, value: &[u8]) -> bool {
        println!("on_header_value [{}]: {:?}", value.len(), str::from_utf8(value).unwrap());
        self.header_value.extend_from_slice(value);
        true
    }

    fn on_headers_finished(&mut self) -> bool {
        println!("on_headers_finished");
        self.headers_finished = true;
        true
    }

    fn on_method(&mut self, method: &[u8]) -> bool {
        println!("on_method [{}]: {:?}", method.len(), str::from_utf8(method).unwrap());
        self.method.extend_from_slice(method);
        true
    }

    fn on_multipart_begin(&mut self) -> bool {
        println!("on_multipart_begin");
        true
    }

    fn on_multipart_data(&mut self, data: &[u8]) -> bool {
        self.multipart_data.extend_from_slice(data);

        for byte in data {
            if !is_visible_7bit!(*byte) {
                println!("on_multipart_data [{}]: *hidden*", data.len());
                return true;
            }
        }

        println!("on_multipart_data [{}]: {:?}", data.len(), str::from_utf8(data).unwrap());
        true
    }

    fn on_status(&mut self, status: &[u8]) -> bool {
        println!("on_status [{}]: {:?}", status.len(), str::from_utf8(status).unwrap());
        self.status.extend_from_slice(status);
        true
    }

    fn on_status_code(&mut self, code: u16) -> bool {
        println!("on_status_code: {}", code);
        self.status_code = code;
        true
    }

    fn on_status_finished(&mut self) -> bool {
        println!("on_status_finished");
        self.status_finished = true;
        true
    }

    fn on_url(&mut self, url: &[u8]) -> bool {
        println!("on_url [{}]: {:?}", url.len(), str::from_utf8(url).unwrap());
        self.url.extend_from_slice(url);
        true
    }

    fn on_url_encoded_field(&mut self, field: &[u8]) -> bool {
        println!("on_url_encoded_field [{}]: {:?}", field.len(), str::from_utf8(field).unwrap());
        self.url_encoded_field.extend_from_slice(field);
        true
    }

    fn on_url_encoded_value(&mut self, value: &[u8]) -> bool {
        println!("on_url_encoded_value [{}]: {:?}", value.len(), str::from_utf8(value).unwrap());
        self.url_encoded_value.extend_from_slice(value);
        true
    }

    fn on_version(&mut self, major: u16, minor: u16) -> bool {
        println!("on_version: {}.{}", major, minor);
        self.version_major = major;
        self.version_minor = minor;
        true
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

    /// Invalid request method on byte `u8`.
    Method(u8),

    /// Invalid multipart data.
    Multipart(u8),

    /// Invalid multipart boundary.
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
            ParserError::ChunkExtensionName(byte) => {
                write!(formatter, "ParserError::ChunkExtensionName(Invalid chunk extension name on byte {})",
                       byte)
            },
            ParserError::ChunkExtensionValue(byte) => {
                write!(formatter, "ParserError::ChunkExtensionValue(Invalid chunk extension value on byte {})",
                       byte)
            },
            ParserError::ChunkLength(byte) => {
                write!(formatter, "ParserError::ChunkLength(Invalid chunk length on byte {})", byte)
            },
            ParserError::CrlfSequence(byte) => {
                write!(formatter, "ParserError::CrlfSequence(Invalid CRLF sequence on byte {})", byte)
            },
            ParserError::Dead => {
                write!(formatter, "ParserError::Dead(Parser is dead)")
            },
            ParserError::HeaderField(byte) => {
                write!(formatter, "ParserError::HeaderField(Invalid header field on byte {})", byte)
            },
            ParserError::HeaderValue(byte) => {
                write!(formatter, "ParserError::HeaderValue(Invalid header value on byte {})", byte)
            },
            ParserError::MaxChunkLength => {
                write!(formatter, "ParserError::MaxChunkLength(Maximum chunk length has been met)")
            },
            ParserError::Method(byte) => {
                write!(formatter, "ParserError::Method(Invalid method on byte {})", byte)
            },
            ParserError::Multipart(byte) => {
                write!(formatter, "ParserError::Multipart(Invalid multipart data on byte {})",
                       byte)
            },
            ParserError::MultipartBoundary(byte) => {
                write!(formatter, "ParserError::MultipartBoundary(Invalid multipart boundary on byte {})",
                       byte)
            },
            ParserError::Status(byte) => {
                write!(formatter, "ParserError::Status(Invalid status on byte {})", byte)
            },
            ParserError::StatusCode(byte) => {
                write!(formatter, "ParserError::StatusCode(Invalid status code on byte {})", byte)
            },
            ParserError::Url(byte) => {
                write!(formatter, "ParserError::Url(Invalid URL on byte {})", byte)
            },
            ParserError::UrlEncodedField(byte) => {
                write!(formatter, "ParserError::UrlEncodedField(Invalid URL encoded field on byte {})",
                       byte)
            },
            ParserError::UrlEncodedValue(byte) => {
                write!(formatter, "ParserError::UrlEncodedValue(Invalid URL encoded value on byte {})",
                       byte)
            },
            ParserError::Version(byte) => {
                write!(formatter, "ParserError::Version(Invalid HTTP version on byte {})", byte)
            }
        }
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserError::ChunkExtensionName(byte) => {
                write!(formatter, "Invalid chunk extension name on byte {}", byte)
            },
            ParserError::ChunkExtensionValue(byte) => {
                write!(formatter, "Invalid chunk extension value on byte {}", byte)
            },
            ParserError::ChunkLength(byte) => {
                write!(formatter, "Invalid chunk length on byte {}", byte)
            },
            ParserError::CrlfSequence(byte) => {
                write!(formatter, "Invalid CRLF sequence on byte {}", byte)
            },
            ParserError::Dead => {
                write!(formatter, "Parser is dead")
            },
            ParserError::HeaderField(byte) => {
                write!(formatter, "Invalid header field on byte {}", byte)
            },
            ParserError::HeaderValue(byte) => {
                write!(formatter, "Invalid header value on byte {}", byte)
            },
            ParserError::MaxChunkLength => {
                write!(formatter, "Maximum chunk length has been met")
            },
            ParserError::Method(byte) => {
                write!(formatter, "Invalid method on byte {}", byte)
            },
            ParserError::Multipart(byte) => {
                write!(formatter, "Invalid multipart data on byte {}", byte)
            },
            ParserError::MultipartBoundary(byte) => {
                write!(formatter, "Invalid multipart boundary on byte {}", byte)
            },
            ParserError::Status(byte) => {
                write!(formatter, "Invalid status on byte {}", byte)
            },
            ParserError::StatusCode(byte) => {
                write!(formatter, "Invalid status code on byte {}", byte)
            },
            ParserError::Url(byte) => {
                write!(formatter, "Invalid URL on byte {}", byte)
            },
            ParserError::UrlEncodedField(byte) => {
                write!(formatter, "Invalid URL encoded field on byte {}", byte)
            },
            ParserError::UrlEncodedValue(byte) => {
                write!(formatter, "Invalid URL encoded value on byte {}", byte)
            },
            ParserError::Version(byte) => {
                write!(formatter, "Invalid HTTP version on byte {}", byte)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Parser types.
///
/// The parser type will be `ParserType::Unknown` until
/// [`Parser::parse_head()`](struct.Parser.html#method.parse_head)
/// is executed, and either of the following has occurred:
///
/// *Requests:*
///
/// [`HttpHandler::on_method()`](trait.HttpHandler.html#method.on_method) has been executed.
///
/// *Responses:*
///
/// [`HttpHandler::on_version()`](trait.HttpHandler.html#method.on_version) has been executed.
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

    /// Parsing status line has finished.
    StatusLineEnd,

    /// Parsing pre-header first line feed.
    PreHeadersLf1,

    /// Parsing pre-header potential second carriage return.
    PreHeadersCr2,

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

    /// Parsing first carriage return after status line or header value.
    HeaderCr1,

    /// Parsing first line feed after status line or header value.
    HeaderLf1,

    /// Parsing second carriage return after status line or header value.
    HeaderCr2,

    /// Parsing second line feed after status line or header value.
    HeaderLf2,

    // ---------------------------------------------------------------------------------------------
    // CHUNKED TRANSFER
    // ---------------------------------------------------------------------------------------------

    /// Parsing chunk length byte 1.
    ChunkLength1,

    /// Parsing chunk length byte 2.
    ChunkLength2,

    /// Parsing chunk length carriage return or semi-colon.
    ChunkLengthCr,

    /// Stripping linear white space before chunk extension name.
    StripChunkExtensionName,

    /// Parsing upper-cased chunk extension.
    UpperChunkExtensionName,

    /// Parsing lower-cased chunk extension.
    LowerChunkExtensionName,

    /// Stripping linear white space before chunk extension value.
    StripChunkExtensionValue,

    /// Parsing chunk extension value.
    ChunkExtensionValue,

    /// Parsing quoted chunk extension value.
    ChunkExtensionQuotedValue,

    /// Parsing potential semi-colon or carriage return after chunk extension quoted value.
    ChunkExtensionQuotedValueFinished,

    /// Parsing escaped chunk extension value.
    ChunkExtensionEscapedValue,

    /// End of chunk extension.
    ChunkExtensionFinished,

    /// Parsing line feed after chunk length.
    ChunkLengthLf,

    /// Parsing chunk data.
    ChunkData,

    /// Parsing carriage return after chunk data.
    ChunkDataNewline1,

    /// Parsing line feed after chunk data.
    ChunkDataNewline2,

    // ---------------------------------------------------------------------------------------------
    // MULTIPART
    // ---------------------------------------------------------------------------------------------

    /// Parsing pre boundary hyphen 1.
    MultipartHyphen1,

    /// Parsing pre boundary hyphen 2.
    MultipartHyphen2,

    /// Parsing multipart boundary.
    MultipartBoundary,

    /// Detecting multipart data parsing mechanism.
    MultipartDetectData,

    /// Parsing multipart data by byte.
    MultipartDataByByte,

    /// Parsing multipart data by content length.
    MultipartDataByLength,

    /// Parsing carriage return after data by length.
    MultipartDataByLengthCr,

    /// Parsing line feed after data by length.
    MultipartDataByLengthLf,

    /// Parsing potential line feed after data by byte.
    MultipartDataByByteLf,

    /// Parsing post boundary carriage return or hyphen.
    MultipartBoundaryCr,

    /// Parsing post boundary line feed.
    MultipartBoundaryLf,

    /// Parsing last boundary second hyphen that indicates end of multipart body.
    MultipartEnd,

    // ---------------------------------------------------------------------------------------------
    // URL ENCODED
    // ---------------------------------------------------------------------------------------------

    /// Parsing URL encoded field.
    UrlEncodedField,

    /// Parsing URL encoded field ampersand or semicolon.
    UrlEncodedFieldAmpersand,

    /// Parsing URL encoded field hex sequence byte 1.
    UrlEncodedFieldHex1,

    /// Parsing URL encoded field hex sequence byte 2.
    UrlEncodedFieldHex2,

    /// Parsing URL encoded field plus sign.
    UrlEncodedFieldPlus,

    /// Parsing URL encoded value.
    UrlEncodedValue,

    /// Parsing URL encoded value hex sequence byte 1.
    UrlEncodedValueHex1,

    /// Parsing URL encoded value hex sequence byte 2.
    UrlEncodedValueHex2,

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
pub trait HttpHandler {
    /// Retrieve the content length.
    ///
    /// **Called From::**
    ///
    /// [`Parser::parse_multipart()`](struct.Parser.html#method.parse_multipart)
    fn content_length(&mut self) -> Option<usize> {
        None
    }

    /// Callback that is executed when body parsing has completed successfully.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_chunked()`](struct.Parser.html#method.parse_chunked)
    ///
    /// Once all chunked data has been parsed.
    ///
    /// [`Parser::parse_multipart()`](struct.Parser.html#method.parse_multipart)
    ///
    /// Once all multipart data has been parsed.
    ///
    /// [`Parser::parse_url_encoded()`](struct.Parser.html#method.parse_url_encoded)
    ///
    /// Once all URL encoded data has been parsed.
    fn on_body_finished(&mut self) -> bool {
        true
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
    /// [`Parser::parse_chunked()`](struct.Parser.html#method.parse_chunked)
    fn on_chunk_data(&mut self, data: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when parsing an individual chunk extension has completed
    /// successfully.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_chunked()`](struct.Parser.html#method.parse_chunked)
    fn on_chunk_extension_finished(&mut self) -> bool {
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
    /// [`Parser::parse_chunked()`](struct.Parser.html#method.parse_chunked)
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
    /// [`Parser::parse_chunked()`](struct.Parser.html#method.parse_chunked)
    fn on_chunk_extension_value(&mut self, value: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when parsing all chunk extensions has completed successfully.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_chunked()`](struct.Parser.html#method.parse_chunked)
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
    /// [`Parser::parse_chunked()`](struct.Parser.html#method.parse_chunked)
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
    /// [`Parser::parse_chunked()`](struct.Parser.html#method.parse_chunked)
    ///
    /// If trailers are present.
    ///
    /// [`Parser::parse_head()`](struct.Parser.html#method.parse_head)
    ///
    /// For standard HTTP headers.
    ///
    /// [`Parser::parse_multipart()`](struct.Parser.html#method.parse_multipart)
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
    /// [`Parser::parse_chunked()`](struct.Parser.html#method.parse_chunked)
    ///
    /// If trailers are supplied.
    ///
    /// [`Parser::parse_head()`](struct.Parser.html#method.parse_head)
    ///
    /// For standard HTTP headers.
    ///
    /// [`Parser::parse_multipart()`](struct.Parser.html#method.parse_multipart)
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
    /// [`Parser::parse_chunked()`](struct.Parser.html#method.parse_chunked)
    ///
    /// If trailers are supplied.
    ///
    /// [`Parser::parse_head()`](struct.Parser.html#method.parse_head)
    ///
    /// For standard HTTP headers.
    ///
    /// [`Parser::parse_multipart()`](struct.Parser.html#method.parse_multipart)
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
    /// [`Parser::parse_head()`](struct.Parser.html#method.parse_head)
    ///
    /// During the initial request line.
    fn on_method(&mut self, method: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a new multipart section has been located.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_multipart()`](struct.Parser.html#method.parse_multipart)
    fn on_multipart_begin(&mut self) -> bool {
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
    /// [`Parser::parse_multipart()`](struct.Parser.html#method.parse_multipart)
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
    /// [`Parser::parse_head()`](struct.Parser.html#method.parse_head)
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
    /// [`Parser::parse_head()`](struct.Parser.html#method.parse_head)
    ///
    /// During the initial response line.
    fn on_status_code(&mut self, code: u16) -> bool {
        true
    }

    /// Callback that is executed when parsing the status line has completed successfully.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called From:**
    ///
    /// [`Parser::parse_head()`](struct.Parser.html#method.parse_head)
    ///
    /// After the status line has been parsed.
    fn on_status_finished(&mut self) -> bool {
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
    /// [`Parser::parse_head()`](struct.Parser.html#method.parse_head)
    ///
    /// During the initial request line.
    fn on_url(&mut self, url: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a URL encoded field has been located.
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
    /// [`Parser::parse_url_encoded()`](struct.Parser.html#method.parse_url_encoded)
    fn on_url_encoded_field(&mut self, field: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a URL encoded value has been located.
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
    /// [`Parser::parse_url_encoded()`](struct.Parser.html#method.parse_url_encoded)
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
    /// [`Parser::parse_head()`](struct.Parser.html#method.parse_head)
    ///
    /// During the initial request or response line.
    fn on_version(&mut self, major: u16, minor: u16) -> bool {
        true
    }
}

// -------------------------------------------------------------------------------------------------

/// Parser context data.
struct ParserContext<'a, T: HttpHandler + 'a> {
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

impl<'a, T: HttpHandler + 'a> ParserContext<'a, T> {
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
pub struct Parser<'a, T: HttpHandler> {
    /// Bit data that stores parser state details, along with HTTP major/minor versions.
    bit_data: u32,

    /// Multipart boundary.
    boundary: Option<&'a [u8]>,

    /// Total byte count processed for headers, and body.
    byte_count: usize,

    /// Length storage for max headers length and chunk length.
    length: usize,

    /// Current state.
    state: ParserState,

    /// Current state function.
    state_function: StateFunction<'a, T>
}

impl<'a, T: HttpHandler> Parser<'a, T> {
    /// Create a new `Parser`.
    ///
    /// The initial state `Parser` is set to is a type detection state that determines if the
    /// stream is a HTTP request or HTTP response.
    pub fn new() -> Parser<'a, T> {
        Parser{ bit_data:       0,
                boundary:       None,
                byte_count:     0,
                length:         0,
                state:          ParserState::StripDetect,
                state_function: Parser::strip_detect }
    }

    /// Retrieve the total byte count processed since the instantiation of `Parser`.
    ///
    /// The byte count is updated when any of the parsing functions completes. This means that if a
    /// call to `byte_count()` is executed from within a callback, it will be accurate within
    /// `stream.len()` bytes. For precise accuracy, the best time to retrieve the byte count is
    /// outside of all callbacks, and outside of the following functions:
    ///
    /// - `parse_chunked()`
    /// - `parse_head()`
    /// - `parse_multipart()`
    /// - `parse_url_encoded()`
    pub fn byte_count(&self) -> usize {
        self.byte_count
    }

    /// Main parser loop.
    #[inline]
    fn parse(&mut self, mut context: &mut ParserContext<T>) -> Result<Success, ParserError> {
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
    /// An implementation of [`HttpHandler`](trait.HttpHandler.html) that provides zero or more of
    /// the callbacks expected by this function.
    ///
    /// **`stream`**
    ///
    /// The stream of data to be parsed.
    ///
    /// # Callbacks
    ///
    /// - [`HttpHandler::on_body_finished()`](trait.HttpHandler.html#method.on_body_finished)
    /// - [`HttpHandler::on_chunk_data()`](trait.HttpHandler.html#method.on_chunk_data)
    /// - [`HttpHandler::on_chunk_extension_finished()`](trait.HttpHandler.html#method.on_chunk_extension_finished)
    /// - [`HttpHandler::on_chunk_extension_name()`](trait.HttpHandler.html#method.on_chunk_extension_name)
    /// - [`HttpHandler::on_chunk_extension_value()`](trait.HttpHandler.html#method.on_chunk_extension_value)
    /// - [`HttpHandler::on_chunk_length()`](trait.HttpHandler.html#method.on_chunk_length)
    /// - [`HttpHandler::on_header_field()`](trait.HttpHandler.html#method.on_header_field)
    /// - [`HttpHandler::on_header_value()`](trait.HttpHandler.html#method.on_header_value)
    /// - [`HttpHandler::on_headers_finished()`](trait.HttpHandler.html#method.on_headers_finished)
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

        self.parse(&mut ParserContext::new(handler, stream))
    }

    /// Parse initial request/response line and all headers.
    ///
    /// # Arguments
    ///
    /// **`handler`**
    ///
    /// An implementation of [`HttpHandler`](trait.HttpHandler.html) that provides zero or more of
    /// the callbacks expected by this function.
    ///
    /// **`stream`**
    ///
    /// The stream of data to be parsed.
    ///
    /// # Callbacks
    ///
    /// *Request & Response:*
    ///
    /// - [`HttpHandler::on_header_field()`](trait.HttpHandler.html#method.on_header_field)
    /// - [`HttpHandler::on_header_value()`](trait.HttpHandler.html#method.on_header_value)
    /// - [`HttpHandler::on_headers_finished()`](trait.HttpHandler.html#method.on_headers_finished)
    /// - [`HttpHandler::on_status_finished()`](trait.HttpHandler.html#method.on_status_finished)
    ///
    /// *Request:*
    ///
    /// - [`HttpHandler::on_method()`](trait.HttpHandler.html#method.on_method)
    /// - [`HttpHandler::on_url()`](trait.HttpHandler.html#method.on_url)
    /// - [`HttpHandler::on_version()`](trait.HttpHandler.html#method.on_version)
    ///
    /// *Response:*
    ///
    /// - [`HttpHandler::on_status()`](trait.HttpHandler.html#method.on_status)
    /// - [`HttpHandler::on_status_code()`](trait.HttpHandler.html#method.on_status_code)
    /// - [`HttpHandler::on_version()`](trait.HttpHandler.html#method.on_version)
    ///
    /// # Errors
    ///
    /// - [`ParserError::CrlfSequence`](enum.ParserError.html#variant.CrlfSequence)
    /// - [`ParserError::HeaderField`](enum.ParserError.html#variant.HeaderField)
    /// - [`ParserError::HeaderValue`](enum.ParserError.html#variant.HeaderValue)
    /// - [`ParserError::Method`](enum.ParserError.html#variant.Method)
    /// - [`ParserError::Status`](enum.ParserError.html#variant.Status)
    /// - [`ParserError::StatusCode`](enum.ParserError.html#variant.StatusCode)
    /// - [`ParserError::Url`](enum.ParserError.html#variant.Url)
    /// - [`ParserError::Version`](enum.ParserError.html#variant.Version)
    #[inline]
    pub fn parse_head(&mut self, handler: &mut T, stream: &[u8])
    -> Result<Success, ParserError> {
        self.parse(&mut ParserContext::new(handler, stream))
    }

    /// Parse multipart data.
    ///
    /// # Arguments
    ///
    /// **`handler`**
    ///
    /// An implementation of [`HttpHandler`](trait.HttpHandler.html) that provides zero or more of
    /// the callbacks expected by this function.
    ///
    /// **`stream`**
    ///
    /// The stream of data to be parsed.
    ///
    /// # Callbacks
    ///
    /// - [`HttpHandler::content_length()`](trait.HttpHandler.html#method.content_length)
    /// - [`HttpHandler::on_body_finished()`](trait.HttpHandler.html#method.on_body_finished)
    /// - [`HttpHandler::on_header_field()`](trait.HttpHandler.html#method.on_header_field)
    /// - [`HttpHandler::on_header_value()`](trait.HttpHandler.html#method.on_header_value)
    /// - [`HttpHandler::on_headers_finished()`](trait.HttpHandler.html#method.on_headers_finished)
    /// - [`HttpHandler::on_multipart_begin()`](trait.HttpHandler.html#method.on_multipart_begin)
    /// - [`HttpHandler::on_multipart_data()`](trait.HttpHandler.html#method.on_multipart_data)
    ///
    /// # Errors
    ///
    /// - [`ParserError::CrlfSequence`](enum.ParserError.html#variant.CrlfSequence)
    /// - [`ParserError::HeaderField`](enum.ParserError.html#variant.HeaderField)
    /// - [`ParserError::HeaderValue`](enum.ParserError.html#variant.HeaderValue)
    #[inline]
    pub fn parse_multipart(&mut self, handler: &mut T, stream: &[u8], boundary: &'a [u8])
    -> Result<Success, ParserError> {
        if self.state == ParserState::StripDetect {
            self.bit_data       = 0;
            self.boundary       = Some(boundary);
            self.length         = 0;
            self.state          = ParserState::MultipartHyphen1;
            self.state_function = Parser::multipart_hyphen1;

            set_flag!(self, F_MULTIPART);

            // lower14 == 1 when we expect a boundary
            set_lower14!(self, 1);
        } else if self.state == ParserState::Finished {
            // already finished
            return Ok(Success::Finished(0));
        }

        self.parse(&mut ParserContext::new(handler, stream))
    }

    /// Parse URL encoded data.
    ///
    /// # Arguments
    ///
    /// **`handler`**
    ///
    /// An implementation of [`HttpHandler`](trait.HttpHandler.html) that provides zero or more of
    /// the callbacks expected by this function.
    ///
    /// **`stream`**
    ///
    /// The stream of data to be parsed.
    ///
    /// **`length`**
    ///
    /// The length of URL encoded data in `stream` that needs parsed.
    ///
    /// # Callbacks
    ///
    /// - [`HttpHandler::on_body_finished()`](trait.HttpHandler.html#method.on_body_finished)
    /// - [`HttpHandler::on_url_encoded_field()`](trait.HttpHandler.html#method.on_url_encoded_field)
    /// - [`HttpHandler::on_url_encoded_value()`](trait.HttpHandler.html#method.on_url_encoded_value)
    ///
    /// # Errors
    ///
    /// - [`ParserError::UrlEncodedField`](enum.ParserError.html#variant.UrlEncodedField)
    /// - [`ParserError::UrlEncodedValue`](enum.ParserError.html#variant.UrlEncodedValue)
    #[inline]
    pub fn parse_url_encoded(&mut self, handler: &mut T, mut stream: &[u8], length: usize)
    -> Result<Success, ParserError> {
        if self.state == ParserState::StripDetect {
            self.bit_data       = 0;
            self.length         = length;
            self.state          = ParserState::UrlEncodedField;
            self.state_function = Parser::url_encoded_field;
        } else if self.state == ParserState::Finished {
            // already finished
            return Ok(Success::Finished(0));
        }

        if self.length < stream.len() {
            // amount of data to process is less than the stream length
            stream = &stream[0..self.length];
        }

        let mut context = ParserContext::new(handler, stream);

        match self.parse(&mut context) {
            Ok(Success::Eos(length)) => {
                if self.length - length == 0 {
                    self.state          = ParserState::BodyFinished;
                    self.state_function = Parser::body_finished;

                    self.parse(&mut context)
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

    /// Reset the parser to its initial state.
    pub fn reset(&mut self) {
        self.bit_data       = 0;
        self.boundary       = None;
        self.byte_count     = 0;
        self.length         = 0;
        self.state          = ParserState::Detect1;
        self.state_function = Parser::detect1;
    }

    /// Resume parsing.
    ///
    /// # Arguments
    ///
    /// **`handler`**
    ///
    /// An implementation of [`HttpHandler`](trait.HttpHandler.html) that provides zero or more of
    /// the callbacks expected by `Parser`.
    ///
    /// **`stream`**
    ///
    /// The stream of data to be parsed.
    #[inline]
    pub fn resume(&mut self, handler: &mut T, stream: &[u8]) -> Result<Success, ParserError> {
        self.parse(&mut ParserContext::new(handler, stream))
    }

    /// Retrieve the current state.
    pub fn state(&self) -> ParserState {
        self.state
    }

    // ---------------------------------------------------------------------------------------------
    // DETECTION STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn strip_detect(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_empty_space!(context,
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
    fn status_line_end(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        set_state!(self, ParserState::PreHeadersLf1, pre_headers_lf1);

        if context.handler.on_status_finished() {
            transition_fast!(self, context);
        }

        exit_callback!(self, context);
    }

    #[inline]
    fn pre_headers_lf1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition_fast!(self, context, ParserState::PreHeadersCr2, pre_headers_cr2);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn pre_headers_cr2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, ParserState::HeaderLf2, header_lf2);
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
                                         ParserState::HeaderLf1, header_lf1);
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
                                             ParserState::HeaderCr1, header_cr1);
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
    fn header_cr1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        if bs_has_bytes!(context, 2) && bs_starts_with2!(context, b"\r\n") {
            bs_jump!(context, 2);

            transition_fast!(self, context, ParserState::HeaderCr2, header_cr2);
        }

        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, ParserState::HeaderLf1, header_lf1);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn header_lf1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition_fast!(self, context, ParserState::HeaderCr2, header_cr2);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn header_cr2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, ParserState::HeaderLf2, header_lf2);
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
    fn header_lf2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            set_flag!(self, F_HEADERS_FINISHED);

            if has_flag!(self, F_CHUNKED) {
                set_state!(self, ParserState::BodyFinished, body_finished);
            } else if has_flag!(self, F_MULTIPART) {
                set_state!(self, ParserState::MultipartDetectData, multipart_detect_data);
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
                set_state!(self, ParserState::StatusLineEnd, status_line_end);

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

        collect_digits16!(context, ParserError::Version, digit, 999, {
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

        collect_digits16!(context, ParserError::Version, digit, 999, {
            set_upper14!(self, digit);

            exit_eos!(self, context);
        });

        if context.byte == b'\r' {
            set_state!(self, ParserState::StatusLineEnd, status_line_end);

            if context.handler.on_version(get_lower14!(self) as u16, digit as u16) {
                transition_fast!(self, context);
            } else {
                exit_callback!(self, context);
            }
        }

        Err(ParserError::Version(context.byte))
    }

    // ---------------------------------------------------------------------------------------------
    // RESPONSE STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn response_version_major(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = get_lower14!(self);

        collect_digits16!(context, ParserError::Version, digit, 999, {
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

        collect_digits16!(context, ParserError::Version, digit, 999, {
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

        collect_digits16!(context, ParserError::StatusCode, digit, 999, {
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
                                         ParserState::StatusLineEnd, status_line_end);
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
            transition_fast!(self, context, ParserState::ChunkLengthCr, chunk_length_cr);
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

        collect_hex64!(context, ParserError::MaxChunkLength, self.length, usize,
            // on end-of-stream
            exit_eos!(self, context)
        );

        bs_replay!(context);

        transition_fast!(self, context, ParserState::ChunkLengthCr, chunk_length_cr);
    }

    #[inline]
    fn chunk_length_cr(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            if self.length == 0 {
                callback_transition_fast!(self, context,
                                          on_chunk_length, self.length,
                                          ParserState::HeaderLf1, header_lf1);
            }

            callback_transition_fast!(self, context,
                                      on_chunk_length, self.length,
                                      ParserState::ChunkLengthLf, chunk_length_lf);
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
    fn chunk_length_lf(&mut self, context: &mut ParserContext<T>)
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
                                             ParserState::StripChunkExtensionValue,
                                             strip_chunk_extension_value);
        } else if context.byte == b'\r' || context.byte == b';' {
            // extension name without a value
            bs_replay!(context);

            callback_transition_fast!(self, context,
                                      on_chunk_extension_name,
                                      ParserState::ChunkExtensionFinished,
                                      chunk_extension_finished);
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

        if context.byte == b'"' {
            transition_fast!(self, context,
                             ParserState::ChunkExtensionQuotedValue, chunk_extension_quoted_value);
        }

        bs_replay!(context);

        callback_transition_fast!(self, context,
                                  on_chunk_extension_value,
                                  ParserState::ChunkExtensionFinished, chunk_extension_finished);
    }

    #[inline]
    fn chunk_extension_quoted_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_quoted_field!(context, ParserError::ChunkExtensionValue,
            // on end-of-stream
            callback_eos_expr!(self, context, on_chunk_extension_value)
        );

        if context.byte == b'"' {
            callback_ignore_transition_fast!(self, context,
                                             on_chunk_extension_value,
                                             ParserState::ChunkExtensionQuotedValueFinished,
                                             chunk_extension_quoted_value_finished);
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
    fn chunk_extension_quoted_value_finished(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b';' || context.byte == b'\r' {
            bs_replay!(context);

            transition!(self, context,
                        ParserState::ChunkExtensionFinished, chunk_extension_finished);
        }

        Err(ParserError::ChunkExtensionValue(context.byte))
    }

    #[inline]
    fn chunk_extension_finished(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            set_state!(self, ParserState::ChunkLengthLf, chunk_length_lf);
        } else {
            set_state!(self, ParserState::UpperChunkExtensionName, upper_chunk_extension_name);
        }

        if context.handler.on_chunk_extension_finished() {
            transition!(self, context);
        }

        exit_callback!(self, context);
    }

    #[inline]
    fn chunk_data(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= self.length {
            // collect remaining chunk data
            bs_collect_length!(context, self.length);

            self.length = 0;

            callback_transition!(self, context,
                                 on_chunk_data,
                                 ParserState::ChunkDataNewline1, chunk_data_newline1);
        }

        // collect remaining stream data
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
        } else if get_lower14!(self) == 0 {
            // we're checking for the boundary within multipart data, but it's not the boundary,
            // so let's send the data to the callback and get back to parsing
            callback_transition!(self, context,
                                 on_multipart_data, &[b'\r', b'\n', context.byte],
                                 ParserState::MultipartDataByByte, multipart_data_by_byte);
        }

        // we're parsing the initial boundary, and it's invalid
        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    fn multipart_hyphen2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'-' {
            transition_fast!(self, context, ParserState::MultipartBoundary, multipart_boundary);
        } else if get_lower14!(self) == 0 {
            // we're checking for the boundary within multipart data, but it's not the boundary,
            // so let's send the data to the callback and get back to parsing
            callback_transition!(self, context,
                                 on_multipart_data, &[b'\r', b'\n', b'-', context.byte],
                                 ParserState::MultipartDataByByte, multipart_data_by_byte);
        }

        // we're parsing the initial boundary, and it's invalid
        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    fn multipart_boundary(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        let (length, callback_data, finished) = {
            let boundary = self.boundary.unwrap();

            let slice = if boundary.len() -
                           get_upper14!(self) as usize <= bs_available!(context) {
                // compare remainder of boundary
                &boundary[get_upper14!(self) as usize..]
            } else {
                // compare remainder of stream
                &boundary[get_upper14!(self) as usize..
                          get_upper14!(self) as usize + bs_available!(context)]
            };

            if bs_starts_with!(context, slice) {
                // matches
                (slice.len(),
                 None,
                 get_upper14!(self) as usize + slice.len() == boundary.len())
            } else {
                // does not match, so we need to provide all the data that has been
                // compared as the boundary up to this point
                let mut v = Vec::with_capacity(// \r\n--
                                               4 as usize +

                                               // old boundary data
                                               get_upper14!(self) as usize);

                v.extend_from_slice(b"\r\n--");
                v.extend_from_slice(&boundary[..get_upper14!(self) as usize]);

                (0, Some(v), false)
            }
        };

        // due to the borrow checker holding 'boundary', we must transition down here
        bs_jump!(context, length);

        if let Some(v) = callback_data {
            // boundary did not match
            if get_lower14!(self) == 0 {
                // reset boundary comparison index
                set_upper14!(self, 0);

                callback_transition!(self, context,
                                     on_multipart_data, &v,
                                     ParserState::MultipartDataByByte, multipart_data_by_byte);
            }

            // we're parsing the initial boundary, and it's invalid
            //
            // there is one caveat to this error:
            //     it will always report the first byte being invalid, even if
            //     it's another byte that did not match, because we're using
            //     bs_starts_with!() vs an individual byte check
            bs_next!(context);

            return Err(ParserError::MultipartBoundary(context.byte));
        }

        // boundary matched
        if finished {
            // boundary comparison finished

            // reset boundary comparison index
            set_upper14!(self, 0);

            transition!(self, context,
                        ParserState::MultipartBoundaryCr, multipart_boundary_cr);
        }

        // boundary comparison not finished
        inc_upper14!(self, length);

        exit_eos!(self, context);
    }

    #[inline]
    fn multipart_boundary_cr(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            set_state!(self, ParserState::PreHeadersLf1, pre_headers_lf1);

            if context.handler.on_multipart_begin() {
                transition!(self, context);
            } else {
                exit_callback!(self, context);
            }
        } else if context.byte == b'-' {
            transition_fast!(self, context,
                             ParserState::MultipartEnd, multipart_end);
        }

        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    fn multipart_boundary_lf(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition!(self, context,
                        ParserState::StripHeaderField, strip_header_field);
        }

        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    fn multipart_detect_data(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        if let Some(length) = context.handler.content_length() {
            self.length = length;

            // expect boundary after data
            set_lower14!(self, 1);

            transition_fast!(self, context,
                             ParserState::MultipartDataByLength,
                             multipart_data_by_length);
        }

        // do not expect boundary since it can be part of the data itself
        set_lower14!(self, 0);

        transition_fast!(self, context,
                         ParserState::MultipartDataByByte,
                         multipart_data_by_byte);
    }

    #[inline]
    fn multipart_data_by_length(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= self.length {
            // collect remaining multipart data
            bs_collect_length!(context, self.length);

            self.length = 0;

            callback_transition!(self, context,
                                 on_multipart_data,
                                 ParserState::MultipartDataByLengthCr, multipart_data_by_length_cr);
        }

        // collect remaining stream data
        self.length -= bs_available!(context);

        bs_collect_length!(context, bs_available!(context));

        callback_transition!(self, context,
                             on_multipart_data,
                             ParserState::MultipartDataByLength, multipart_data_by_length);
    }

    #[inline]
    fn multipart_data_by_length_cr(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context,
                             ParserState::MultipartDataByLengthLf, multipart_data_by_length_lf);
        }

        // this state is only used after multipart_data_by_length, so we can error if we don't
        // find the carriage return
        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    fn multipart_data_by_length_lf(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition_fast!(self, context,
                             ParserState::MultipartHyphen1, multipart_hyphen1);
        }

        // this state is only used after multipart_data_by_length, so we can error if we don't
        // find the carriage return
        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    fn multipart_data_by_byte(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        bs_collect_until!(context,
            // collect bytes until
            context.byte == b'\r',

            // on end-of-stream
            callback_eos_expr!(self, context, on_multipart_data)
        );

        callback_ignore_transition_fast!(self, context,
                                         on_multipart_data,
                                         ParserState::MultipartDataByByteLf, multipart_data_by_byte_lf)
    }

    #[inline]
    fn multipart_data_by_byte_lf(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition_fast!(self, context,
                             ParserState::MultipartHyphen1, multipart_hyphen1);
        }

        callback_transition!(self, context,
                             on_multipart_data, &[b'\r', context.byte],
                             ParserState::MultipartDataByByte, multipart_data_by_byte);
    }

    #[inline]
    fn multipart_end(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'-' {
            transition!(self, context,
                        ParserState::BodyFinished, body_finished);
        }

        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    fn url_encoded_field(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_visible!(context, ParserError::UrlEncodedField,
            // stop on these bytes
               context.byte == b'='
            || context.byte == b'%'
            || context.byte == b'&'
            || context.byte == b';'
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
                                                 ParserState::UrlEncodedFieldHex1,
                                                 url_encoded_field_hex1);
            },
            b'&' | b';' => {
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
    fn url_encoded_field_hex1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        set_upper14!(self, if is_digit!(context.byte) {
            (context.byte - b'0') << 4
        } else if b'@' < context.byte && context.byte < b'G' {
            (context.byte - 0x37) << 4
        } else if b'`' < context.byte && context.byte < b'g' {
            (context.byte - 0x57) << 4
        } else {
            return Err(ParserError::UrlEncodedField(context.byte));
        });

        transition_fast!(self, context,
                         ParserState::UrlEncodedFieldHex2, url_encoded_field_hex2);
    }

    #[inline]
    fn url_encoded_field_hex2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        set_lower14!(self, if is_digit!(context.byte) {
            context.byte - b'0'
        } else if b'@' < context.byte && context.byte < b'G' {
            context.byte - 0x37
        } else if b'`' < context.byte && context.byte < b'g' {
            context.byte - 0x57
        } else {
            return Err(ParserError::UrlEncodedField(context.byte));
        });

        callback_transition!(self, context,
                             on_url_encoded_field,
                             &[(get_upper14!(self) | get_lower14!(self)) as u8],
                             ParserState::UrlEncodedField, url_encoded_field);
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
            || context.byte == b';'
            || context.byte == b'+'
            || context.byte == b'=',

            // on end-of-stream
            callback_eos_expr!(self, context, on_url_encoded_value)
        );

        match context.byte {
            b'%' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_value,
                                                 ParserState::UrlEncodedValueHex1, url_encoded_value_hex1);
            },
            b'&' | b';' => {
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
    fn url_encoded_value_hex1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        set_upper14!(self, if is_digit!(context.byte) {
            (context.byte - b'0') << 4
        } else if b'@' < context.byte && context.byte < b'G' {
            (context.byte - 0x37) << 4
        } else if b'`' < context.byte && context.byte < b'g' {
            (context.byte - 0x57) << 4
        } else {
            return Err(ParserError::UrlEncodedValue(context.byte));
        });

        transition_fast!(self, context,
                         ParserState::UrlEncodedValueHex2, url_encoded_value_hex2);
    }

    #[inline]
    fn url_encoded_value_hex2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        set_lower14!(self, if is_digit!(context.byte) {
            context.byte - b'0'
        } else if b'@' < context.byte && context.byte < b'G' {
            context.byte - 0x37
        } else if b'`' < context.byte && context.byte < b'g' {
            context.byte - 0x57
        } else {
            return Err(ParserError::UrlEncodedValue(context.byte));
        });

        callback_transition!(self, context,
                             on_url_encoded_value,
                             &[(get_upper14!(self) | get_lower14!(self)) as u8],
                             ParserState::UrlEncodedValue, url_encoded_value);
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
