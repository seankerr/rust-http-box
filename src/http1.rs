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

// -------------------------------------------------------------------------------------------------

// All 28 bits mask.
const ALL28_MASK: u32 = 0xFFFFFFF;

// All 28 bits shift.
const ALL28_SHIFT: u32 = 4;

// State flag mask.
const FLAG_MASK: u32 = 0x4;

// State flag shift.
const FLAG_SHIFT: u8 = 0;

// Lower 14 bits mask.
const LOWER14_MASK: u32 = 0x3FFF;

// Lower 14 bits shift.
const LOWER14_SHIFT: u8 = 4;

// Upper 14 bits mask.
const UPPER14_MASK: u32 = 0x3FFF;

// Upper 14 bits shift.
const UPPER14_SHIFT: u8 = 18;

// -------------------------------------------------------------------------------------------------
// FLAGS
// -------------------------------------------------------------------------------------------------

// these flags are actually bit shift amounts, not typical power of 2 flag values themselves

// Parsing chunked transfer encoding.
const F_CHUNKED: u8 = 0;

// Parsing multipart data.
const F_MULTIPART: u8 = 1;

// Parsing request.
const F_REQUEST: u8 = 2;

// Parsing response.
const F_RESPONSE: u8 = 3;

// -------------------------------------------------------------------------------------------------
// BIT DATA MACROS
// -------------------------------------------------------------------------------------------------

// Retrieve all 28 bits.
macro_rules! get_all28 {
    ($parser:expr) => ({
        ($parser.bit_data >> ALL28_SHIFT) & ALL28_MASK
    });
}


// Retrieve the lower 14 bits.
macro_rules! get_lower14 {
    ($parser:expr) => ({
        ($parser.bit_data >> LOWER14_SHIFT) & LOWER14_MASK
    });
}

// Retrieve the upper 14 bits.
macro_rules! get_upper14 {
    ($parser:expr) => ({
        ($parser.bit_data >> UPPER14_SHIFT) & UPPER14_MASK
    });
}

// Indicates that a state flag is set.
macro_rules! has_flag {
    ($parser:expr, $flag:expr) => ({
        (($parser.bit_data >> FLAG_SHIFT) & FLAG_MASK) & (1 << $flag) == (1 << $flag)
    });
}

// Set a state flag.
macro_rules! set_flag {
    ($parser:expr, $flag:expr) => ({
        $parser.bit_data |= ((1 << $flag) & FLAG_MASK) << FLAG_SHIFT;
    });
}

// Set all 28 bits.
macro_rules! set_all28 {
    ($parser:expr, $bits:expr) => ({
        let bits = $bits as u32;

        $parser.bit_data &= !(ALL28_MASK << ALL28_SHIFT);
        $parser.bit_data |= bits << ALL28_SHIFT;
    });
}

// Set the lower 14 bits.
macro_rules! set_lower14 {
    ($parser:expr, $bits:expr) => ({
        let bits = $bits as u32;

        $parser.bit_data &= !(LOWER14_MASK << LOWER14_SHIFT);
        $parser.bit_data |= bits << LOWER14_SHIFT;
    });
}

// Set the upper 14 bits.
macro_rules! set_upper14 {
    ($parser:expr, $bits:expr) => ({
        let bits = $bits as u32;

        $parser.bit_data &= !(UPPER14_MASK << UPPER14_SHIFT);
        $parser.bit_data |= bits << UPPER14_SHIFT;
    });
}

// Unset a state flag.
macro_rules! unset_flag {
    ($parser:expr, $flag:expr) => ({
        $parser.bit_data &= !(((1 << $flag) & FLAG_MASK) << FLAG_SHIFT);
    });
}

// -------------------------------------------------------------------------------------------------

// Parser state function type.
type StateFunction<'a, T> = fn(&mut Parser<'a, T>, &mut ParserContext<T>)
    -> Result<ParserValue, ParserError>;

// -------------------------------------------------------------------------------------------------

/// Parser error messages.
#[derive(Clone,Copy,PartialEq)]
pub enum ParserError {
    /// Invalid chunk extension name on byte `u8`.
    ChunkExtensionName(u8),

    /// Invalid chunk extension value on byte `u8`.
    ChunkExtensionValue(u8),

    /// Invalid chunk size on byte `u8`.
    ChunkSize(u8),

    /// Invalid CRLF sequence on byte `u8`.
    CrlfSequence(u8),

    /// Parsing has failed.
    Dead,

    /// Invalid header field on byte `u8`.
    HeaderField(u8),

    /// Invalid header value on byte `u8`.
    HeaderValue(u8),

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
            ParserError::ChunkSize(ref byte) => {
                write!(formatter, "ParserError::ChunkSize(Invalid chunk size on byte {})", byte)
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
            ParserError::ChunkSize(ref byte) => {
                write!(formatter, "Invalid chunk size on byte {}", byte)
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
/// These states are in the order that they are processed.
#[derive(Clone,Copy,Debug,PartialEq)]
#[repr(u8)]
pub enum State {
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

    /// Parsing header field.
    HeaderField,

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
    // BODY
    // ---------------------------------------------------------------------------------------------

    /// Parsing chunk size byte 1.
    ChunkSize1,

    /// Parsing chunk size byte 2.
    ChunkSize2,

    /// Parsing chunk size end (when chunk size is 0).
    ChunkSizeEnd,

    /// Parsing chunk extension name.
    ChunkExtensionName,

    /// Parsing chunk extension value.
    ChunkExtensionValue,

    /// Parsing quoted chunk extension value.
    ChunkExtensionQuotedValue,

    /// Parsing escaped chunk extension value.
    ChunkExtensionEscapedValue,

    /// Parsing potential semi-colon or carriage return after chunk extension quoted value.
    ChunkExtensionSemiColon,

    /// Parsing line feed after chunk size.
    ChunkSizeNewline,

    /// Parsing chunk data.
    ChunkData,

    /// Parsing carriage return after chunk data.
    ChunkDataNewline1,

    /// Parsing line feed after chunk data.
    ChunkDataNewline2,

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

    /// Parsing carriage return at end of message.
    FinishedNewline1,

    /// Parsing line feed at end of message.
    FinishedNewline2,

    /// Parsing has finished successfully.
    Finished
}

// -------------------------------------------------------------------------------------------------

/// Type that handles HTTP parser events.
#[allow(unused_variables)]
pub trait Http1Handler {
    /// Retrieve the multipart boundary.
    fn get_boundary(&mut self) -> Option<&[u8]> {
        None
    }

    /// Callback that is executed when a chunk of data has been parsed.
    ///
    /// This may be executed multiple times in order to supply the entire chunk.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_chunk_data(&mut self, data: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a chunk extension name has been located.
    ///
    /// This may be executed multiple times in order to supply the entire chunk extension name.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_chunk_extension_name(&mut self, name: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a chunk extension value has been located.
    ///
    /// This may be executed multiple times in order to supply the entire chunk extension value.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_chunk_extension_value(&mut self, value: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a chunk size has been located.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_chunk_size(&mut self, size: u32) -> bool {
        true
    }

    /// Callback that is executed when a header field has been located.
    ///
    /// This may be executed multiple times in order to supply the entire header field.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_header_field(&mut self, field: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a header value has been located.
    ///
    /// This may be executed multiple times in order to supply the entire header value.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_header_value(&mut self, value: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when header parsing has completed successfully.
    fn on_headers_finished(&mut self) {
    }

    /// Callback that is executed when a request method has been located.
    ///
    /// This may be executed multiple times in order to supply the entire method.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_method(&mut self, method: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when multipart data has been located.
    ///
    /// This may be executed multiple times in order to supply the entire piece of data.
    fn on_multipart_data(&mut self, data: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a response status has been located.
    ///
    /// This may be executed multiple times in order to supply the entire status.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_status(&mut self, status: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a response status code has been located.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_status_code(&mut self, code: u16) -> bool {
        true
    }

    /// Callback that is executed when a request URL/path has been located.
    ///
    /// This may be executed multiple times in order to supply the entire URL/path.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url(&mut self, url: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a URL encoded field or query string field has been located.
    ///
    /// This may be executed multiple times in order to supply the entire field.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_encoded_field(&mut self, field: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a URL encoded value or query string value has been located.
    ///
    /// This may be executed multiple times in order to supply the entire value.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_encoded_value(&mut self, value: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when the HTTP major version has been located.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_version(&mut self, major: u16, minor: u16) -> bool {
        true
    }
}

// -------------------------------------------------------------------------------------------------

// Parser context data.
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
    // Bit data that stores parser bit details.
    //
    // Bits 1-4: State flags that are checked when states have a dual purpose, such as when header
    //           parsing states also parse chunk encoding trailers.
    // Macros:   has_flag!(), set_flag!(), unset_flag!()
    //
    // Bits 5-32: Used to store various numbers depending on state. Content length, chunk size,
    //            HTTP major/minor versions are all stored in here. Depending on macro used, more
    //            bits are accessible.
    // Macros:    get_lower8!(), set_lower8!()   -- lower 8 bits
    //            get_mid8!(), set_mid8!()       -- mid 8 bits
    //            get_lower16!(), set_lower16!() -- lower 16 bits
    //                                              (when not using the lower8/mid8 macros)
    //            get_upper40!(), set_upper40!() -- upper 40 bits
    bit_data: u32,

    // Total byte count processed for headers, and body.
    // Once the headers are finished processing, this is reset to 0 to track the body length.
    byte_count: usize,

    // Current state.
    state: State,

    // Current state function.
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
                state:          State::StripDetect,
                state_function: Parser::strip_detect }
    }

    /// Retrieve the total byte count processed since the instantiation of `Parser`.
    ///
    /// The byte count is updated when any of the parsing functions completes. This means that if a
    /// call to `get_byte_count()` is executed from within a callback, it will be accurate within
    /// `stream.len()` bytes. For precise accuracy, the best time to retrieve the byte count is
    /// outside of all callbacks, and outside of the following functions:
    ///
    /// - [Http1Handler::parse_chunked()](#method.parse_chunked)
    /// - [Http1Handler::parse_multipart()](#method.parse_multipart)
    /// - [Http1Handler::parse_url_encoded()](#method.parse_url_encoded)
    pub fn get_byte_count(&self) -> usize {
        self.byte_count
    }

    /// Retrieve the current state.
    pub fn get_state(&self) -> State {
        self.state
    }

    /// Retrieve the parser type.
    ///
    /// The parser type will be [ParserType::Unknown](enum.ParserType.html#variant.Unknown) until
    /// `parse_headers()` is executed, and either of the following has occurred:
    ///
    ///  - For requests: [Http1Handler::on_method()](trait.Http1Handler.html#method.on_method) has
    ///    been executed
    ///  - For responses: [Http1Handler::on_version()](trait.Http1Handler.html#method.on_version)
    ///    has been executed
    pub fn get_type(&self) -> ParserType {
        if has_flag!(self, F_REQUEST) {
            ParserType::Request
        } else if has_flag!(self, F_RESPONSE) {
            ParserType::Response
        } else {
            ParserType::Unknown
        }
    }

    // Main parser loop.
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
                        self.state = State::Finished;
                    }

                    return Ok(success);
                },
                Err(error) => {
                    self.byte_count     += context.stream_index;
                    self.state           = State::Dead;
                    self.state_function  = Parser::dead;

                    return Err(error);
                }
            }
        }
    }

    /// Parse chunked transfer encoded data.
    ///
    /// If [Success::Callback](enum.Success.html#variant.Callback) is returned, a callback returned
    /// `false` and parsing exited prematurely. This can be treated the same as
    /// [Success::Eos](enum.Success.html#variant.Eos).
    ///
    /// If [Success::Eos](enum.Success.html#variant.Eos) is returned, additional `stream` data is
    /// expected. You must call `parse_chunked()` again until
    /// [Success::Finished](enum.Success.html#variant.Finished) is returned.
    ///
    /// If [Success::Finished](enum.Success.html#variant.Finished) is returned, parsing has finished
    /// successfully.
    ///
    /// **The following callbacks are used by `parse_chunked()`:**
    ///
    /// - [Http1Handler::on_chunk_data()](trait.Http1Handler.html#method.on_chunk_data)
    /// - [Http1Handler::on_chunk_extension_name()](trait.Http1Handler.html#method.on_chunk_extension_name)
    /// - [Http1Handler::on_chunk_extension_value()](trait.Http1Handler.html#method.on_chunk_extension_value)
    /// - [Http1Handler::on_chunk_size()](trait.Http1Handler.html#method.on_chunk_size)
    /// - [Http1Handler::on_header_field()](trait.Http1Handler.html#method.on_header_field)
    /// - [Http1Handler::on_header_value()](trait.Http1Handler.html#method.on_header_value)
    #[inline]
    pub fn parse_chunked(&mut self, handler: &mut T, stream: &[u8])
    -> Result<Success, ParserError> {
        if self.state == State::StripDetect {
            set_flag!(self, F_CHUNKED);
            set_all28!(self, 0);

            self.state          = State::ChunkSize1;
            self.state_function = Parser::chunk_size1;
        }

        self.parse(handler, stream)
    }

    /// Parse initial request/response line and all headers.
    ///
    /// If [Success::Callback](enum.Success.html#variant.Callback) is returned, a callback returned
    /// `false` and parsing exited prematurely. This can be treated the same as
    /// [Success::Eos](enum.Success.html#variant.Eos).
    ///
    /// If [Success::Eos](enum.Success.html#variant.Eos) is returned, additional `stream` data is
    /// expected. You must call `parse_headers()` again until
    /// [Success::Finished](enum.Success.html#variant.Finished) is returned.
    ///
    /// If [Success::Finished](enum.Success.html#variant.Finished) is returned, parsing has finished
    /// successfully.
    ///
    /// **The following callbacks are used by `parse_headers()`:**
    ///
    /// - [Http1Handler::on_header_field()](trait.Http1Handler.html#method.on_header_field)
    /// - [Http1Handler::on_header_value()](trait.Http1Handler.html#method.on_header_value)
    ///
    /// **Request callbacks:**
    ///
    /// - [Http1Handler::on_method()](trait.Http1Handler.html#method.on_method)
    /// - [Http1Handler::on_url()](trait.Http1Handler.html#method.on_url)
    /// - [Http1Handler::on_version()](trait.Http1Handler.html#method.on_version)
    ///
    /// **Response callbacks:**
    ///
    /// - [Http1Handler::on_status()](trait.Http1Handler.html#method.on_status)
    /// - [Http1Handler::on_status_code()](trait.Http1Handler.html#method.on_status_code)
    /// - [Http1Handler::on_version()](trait.Http1Handler.html#method.on_version)
    #[inline]
    pub fn parse_headers(&mut self, handler: &mut T, stream: &[u8])
    -> Result<Success, ParserError> {
        self.parse(handler, stream)
    }

    /// Parse multipart data.
    ///
    /// If [Success::Callback](enum.Success.html#variant.Callback) is returned, a callback returned
    /// `false` and parsing exited prematurely. This can be treated the same as
    /// [Success::Eos](enum.Success.html#variant.Eos).
    ///
    /// If [Success::Eos](enum.Success.html#variant.Eos) is returned, additional `stream` data is
    /// expected. You must call `parse_multipart()` again until
    /// [Success::Finished](enum.Success.html#variant.Finished) is returned.
    ///
    /// If [Success::Finished](enum.Success.html#variant.Finished) is returned, parsing has finished
    /// successfully.
    ///
    /// **The following callbacks are used by `parse_multipart()`:**
    ///
    /// - [Http1Handler::get_boundary()](trait.Http1Handler.html#method.get_boundary)
    /// - [Http1Handler::on_header_field()](trait.Http1Handler.html#method.on_header_field)
    /// - [Http1Handler::on_header_value()](trait.Http1Handler.html#method.on_header_value)
    /// - [Http1Handler::on_multipart_data()](trait.Http1Handler.html#method.on_multipart_data)
    #[inline]
    pub fn parse_multipart(&mut self, handler: &mut T, stream: &[u8])
    -> Result<Success, ParserError> {
        if self.state == State::StripDetect {
            self.state          = State::MultipartHyphen1;
            self.state_function = Parser::multipart_hyphen1;
        }

        self.parse(handler, stream)
    }

    /// Parse URL encoded data.
    ///
    /// If [Success::Callback](enum.Success.html#variant.Callback) is returned, a callback returned
    /// `false` and parsing exited prematurely. This can be treated the same as
    /// [Success::Eos](enum.Success.html#variant.Eos).
    ///
    /// If [Success::Eos](enum.Success.html#variant.Eos) is returned, additional `stream` data is
    /// expected. You must call `parse_url_encoded()` again until
    /// [Success::Finished](enum.Success.html#variant.Finished) is returned.
    ///
    /// If [Success::Finished](enum.Success.html#variant.Finished) is returned, parsing has finished
    /// successfully.
    ///
    /// **The following callbacks are used by `parse_url_encoded()`:**
    ///
    /// - [Http1Handler::on_url_encoded_field()](trait.Http1Handler.html#method.on_url_encoded_field)
    /// - [Http1Handler::on_url_encoded_value()](trait.Http1Handler.html#method.on_url_encoded_value)
    #[inline]
    pub fn parse_url_encoded(&mut self, handler: &mut T, stream: &[u8])
    -> Result<Success, ParserError> {
        if self.state == State::StripDetect {
            self.state          = State::UrlEncodedField;
            self.state_function = Parser::url_encoded_field;
        }

        self.parse(handler, stream)
    }

    /// Reset the parser back to its initial state.
    pub fn reset(&mut self) {
        self.bit_data       = 0;
        self.byte_count     = 0;
        self.state          = State::Detect1;
        self.state_function = Parser::detect1;
    }

    // ---------------------------------------------------------------------------------------------
    // DETECTION STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn strip_detect(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);

        transition_fast!(self, context, State::Detect1, detect1);
    }

    #[inline]
    fn detect1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        macro_rules! version {
            ($major:expr, $minor:expr, $length:expr) => ({
                set_flag!(self, F_RESPONSE);
                bs_jump!(context, $length);
                set_state!(self, State::StripResponseStatusCode, strip_response_status_code);

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
                                     State::ResponseVersionMajor, response_version_major);
                }
            } else {
                bs_jump!(context, 1);

                transition_fast!(self, context, State::Detect2, detect2);
            }
        }

        // this is a request
        transition_fast!(self, context, State::RequestMethod, request_method);
    }

    #[inline]
    fn detect2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'T' || context.byte == b't' {
            transition_fast!(self, context, State::Detect3, detect3);
        }

        // since we're in a detection state and didn't know until right here that we're moving from
        // detection -> request, we need to manually submit the first n bytes of the of the request
        // method, and the request method state will do the rest of the work for us
        bs_replay!(context);

        callback_transition_fast!(self, context, on_method, b"H",
                                  State::RequestMethod, request_method);
    }

    #[inline]
    fn detect3(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'T' || context.byte == b't' {
            transition_fast!(self, context, State::Detect4, detect4);
        }

        // since we're in a detection state and didn't know until right here that we're moving from
        // detection -> request, we need to manually submit the first n bytes of the of the request
        // method, and the request method state will do the rest of the work for us
        bs_replay!(context);

        callback_transition_fast!(self, context, on_method, b"HT",
                                  State::RequestMethod, request_method);
    }

    #[inline]
    fn detect4(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'P' || context.byte == b'p' {
            transition_fast!(self, context, State::Detect5, detect5);
        }

        // since we're in a detection state and didn't know until right here that we're moving from
        // detection -> request, we need to manually submit the first n bytes of the of the request
        // method, and the request method state will do the rest of the work for us
        bs_replay!(context);

        callback_transition_fast!(self, context, on_method, b"HTT",
                                  State::RequestMethod, request_method);
    }

    #[inline]
    fn detect5(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'/' {
            set_flag!(self, F_RESPONSE);
            set_all28!(self, 0);

            transition_fast!(self, context, State::ResponseVersionMajor, response_version_major);
        }

        // since we're in a detection state and didn't know until right here that we're moving from
        // detection -> request, we need to manually submit the first n bytes of the of the request
        // method, and the request method state will do the rest of the work for us
        bs_replay!(context);

        callback_transition_fast!(self, context, on_method, b"HTTP",
                                  State::RequestMethod, request_method);
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
            transition_fast!(self, context, State::PreHeaders2, pre_headers2);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn pre_headers2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, State::Newline4, newline4);
        } else {
            bs_replay!(context);

            transition_fast!(self, context, State::StripHeaderField, strip_header_field);
        }
    }

    #[inline]
    fn strip_header_field(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);

        transition_fast!(self, context, State::FirstHeaderField, first_header_field);
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
                                          State::StripHeaderValue, strip_header_value);
            });
        }

        if bs_has_bytes!(context, 26) {
            // have enough bytes to compare common header fields immediately, without collecting
            // individual tokens
            if context.byte == b'C' {
                if bs_starts_with11!(context, b"Connection:") {
                    field!(b"Connection", 11);
                } else if bs_starts_with13!(context, b"Content-Type:") {
                    field!(b"Content-Type", 13);
                } else if bs_starts_with15!(context, b"Content-Length:") {
                    field!(b"Content-Length", 15);
                } else if bs_starts_with7!(context, b"Cookie:") {
                    field!(b"Cookie", 7);
                } else if bs_starts_with14!(context, b"Cache-Control:") {
                    field!(b"Cache-Control", 14);
                } else if bs_starts_with24!(context, b"Content-Security-Policy:") {
                    field!(b"Content-Security-Policy", 24);
                }
            } else if context.byte == b'A' {
                if bs_starts_with7!(context, b"Accept:") {
                    field!(b"Accept", 7);
                } else if bs_starts_with15!(context, b"Accept-Charset:") {
                    field!(b"Accept-Charset", 15);
                } else if bs_starts_with16!(context, b"Accept-Encoding:") {
                    field!(b"Accept-Encoding", 16);
                } else if bs_starts_with16!(context, b"Accept-Language:") {
                    field!(b"Accept-Language", 16);
                } else if bs_starts_with14!(context, b"Authorization:") {
                    field!(b"Authorization", 14);
                }
            } else if context.byte == b'L' {
                if bs_starts_with9!(context, b"Location:") {
                    field!(b"Location", 9);
                } else if bs_starts_with14!(context, b"Last-Modified:") {
                    field!(b"Last-Modified", 14);
                }
            } else if bs_starts_with7!(context, b"Pragma:") {
                field!(b"Pragma", 7);
            } else if bs_starts_with11!(context, b"Set-Cookie:") {
                field!(b"Set-Cookie", 11);
            } else if bs_starts_with18!(context, b"Transfer-Encoding:") {
                field!(b"Transfer-Encoding", 18);
            } else if context.byte == b'U' {
                if bs_starts_with11!(context, b"User-Agent:") {
                    field!(b"User-Agent", 11);
                } else if bs_starts_with8!(context, b"Upgrade:") {
                    field!(b"Upgrade", 8);
                }
            } else if context.byte == b'X' {
                if bs_starts_with13!(context, b"X-Powered-By:") {
                    field!(b"X-Powered-By", 13);
                } else if bs_starts_with16!(context, b"X-Forwarded-For:") {
                    field!(b"X-Forwarded-For", 16);
                } else if bs_starts_with17!(context, b"X-Forwarded-Host:") {
                    field!(b"X-Forwarded-Host", 17);
                } else if bs_starts_with17!(context, b"X-XSS-Protection:") {
                    field!(b"X-XSS-Protection", 17);
                } else if bs_starts_with13!(context, b"X-WebKit-CSP:") {
                    field!(b"X-WebKit-CSP", 13);
                } else if b"X-Content-Security-Policy:" == bs_peek!(context, 26) {
                    field!(b"X-Content-Security-Policy", 26);
                }
            } else if bs_starts_with17!(context, b"WWW-Authenticate:") {
                field!(b"WWW-Authenticate", 17);
            }
        }

        transition_fast!(self, context, State::HeaderField, header_field);
    }

    #[inline]
    fn header_field(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_tokens!(context, ParserError::HeaderField,
            // stop on these bytes
            context.byte == b':',

            // on end-of-stream
            callback_eos_expr!(self, context, on_header_field)
        );

        callback_ignore_transition_fast!(self, context,
                                         on_header_field,
                                         State::StripHeaderValue, strip_header_value);
    }

    #[inline]
    fn strip_header_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);
        bs_next!(context);

        if context.byte == b'"' {
            transition_fast!(self, context, State::HeaderQuotedValue, header_quoted_value);
        }

        bs_replay!(context);

        transition_fast!(self, context, State::HeaderValue, header_value);
    }

    #[inline]
    fn header_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_header_value!(context, ParserError::HeaderValue,
            callback_eos_expr!(self, context, on_header_value)
        );

        callback_ignore_transition_fast!(self, context,
                                         on_header_value,
                                         State::Newline2, newline2);
    }

    #[inline]
    fn header_quoted_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_quoted_value!(self, context, ParserError::HeaderValue, on_header_value);

        if context.byte == b'"' {
            callback_ignore_transition_fast!(self, context,
                                             on_header_value,
                                             State::Newline1, newline1);
        } else {
            callback_ignore_transition_fast!(self, context,
                                             on_header_value,
                                             State::HeaderEscapedValue, header_escaped_value);
        }
    }

    #[inline]
    fn header_escaped_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        callback_transition!(self, context,
                             on_header_value, &[context.byte],
                             State::HeaderQuotedValue, header_quoted_value);
    }

    #[inline]
    fn newline1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        if bs_has_bytes!(context, 2) && bs_starts_with2!(context, b"\r\n") {
            transition_fast!(self, context, State::Newline3, newline3);
        }

        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, State::Newline2, newline2);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn newline2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition_fast!(self, context, State::Newline3, newline3);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn newline3(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, State::Newline4, newline4);
        } else if context.byte == b' ' || context.byte == b'\t' {
            // multiline header value
            callback_transition!(self, context,
                                 on_header_value, b" ",
                                 State::StripHeaderValue, strip_header_value);
        } else {
            bs_replay!(context);
            transition!(self, context, State::StripHeaderField, strip_header_field);
        }
    }

    #[inline]
    fn newline4(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            context.handler.on_headers_finished();

            transition_fast!(self, context, State::Finished, finished);
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
                                          State::StripRequestUrl, strip_request_url);
            );
        }

        set_flag!(self, F_REQUEST);

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
                                  State::StripRequestUrl, strip_request_url);
    }

    #[inline]
    fn strip_request_url(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);

        transition_fast!(self, context, State::RequestUrl, request_url);
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
                                  State::StripRequestHttp, strip_request_http);
    }

    #[inline]
    fn strip_request_http(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);

        transition_fast!(self, context, State::RequestHttp1, request_http1);
    }

    #[inline]
    fn request_http1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        macro_rules! version {
            ($major:expr, $minor:expr, $length:expr) => (
                bs_jump!(context, $length);
                set_state!(self, State::PreHeaders1, pre_headers1);

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
            transition_fast!(self, context, State::RequestHttp2, request_http2);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_http2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'T' || context.byte == b't' {
            transition_fast!(self, context, State::RequestHttp3, request_http3);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_http3(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'T' || context.byte == b't' {
            transition_fast!(self, context, State::RequestHttp4, request_http4);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_http4(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'P' || context.byte == b'p' {
            transition_fast!(self, context, State::RequestHttp5, request_http5);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_http5(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'/' {
            set_all28!(self, 0);

            transition_fast!(self, context, State::RequestVersionMajor, request_version_major);
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
            transition_fast!(self, context, State::RequestVersionMinor, request_version_minor);
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

        set_state!(self, State::PreHeaders1, pre_headers1);

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
            transition_fast!(self, context, State::ResponseVersionMinor, response_version_minor);
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

        set_state!(self, State::StripResponseStatusCode, strip_response_status_code);

        if context.handler.on_version(get_lower14!(self) as u16, digit as u16) {
            transition_fast!(self, context);
        } else {
            exit_callback!(self, context);
        }
    }

    #[inline]
    fn strip_response_status_code(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);
        bs_next!(context);

        if !is_digit!(context.byte) {
            return Err(ParserError::StatusCode(context.byte));
        }

        bs_replay!(context);

        set_lower14!(self, 0);

        transition_fast!(self, context, State::ResponseStatusCode, response_status_code);
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
        set_state!(self, State::StripResponseStatus, strip_response_status);

        if context.handler.on_status_code(digit as u16) {
            transition_fast!(self, context);
        } else {
            exit_callback!(self, context);
        }
    }

    #[inline]
    fn strip_response_status(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);

        transition_fast!(self, context, State::ResponseStatus, response_status);
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
                                         State::PreHeaders1, pre_headers1);
    }

    // ---------------------------------------------------------------------------------------------
    // BODY STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn chunk_size1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'0' {
            transition_fast!(self, context, State::ChunkSizeEnd, chunk_size_end);
        } else if !is_hex!(context.byte) {
            return Err(ParserError::ChunkSize(context.byte));
        }

        bs_replay!(context);

        transition_fast!(self, context, State::ChunkSize2, chunk_size2);
    }

    #[inline]
    fn chunk_size2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        match hex_to_byte(&[context.byte]) {
            Some(byte) => {
                if (get_all28!(self) << 4) + byte as u32 > 0xFFFFFFF {
                    // beyond maximum chunk size (28 bits)
                    return Err(ParserError::ChunkSize(context.byte));
                }

                set_all28!(self, get_all28!(self) << 4);
                set_all28!(self, get_all28!(self) + byte as u32);

                transition!(self, context, State::ChunkSize2, chunk_size2);
            },
            None => {
                bs_replay!(context);

                transition_fast!(self, context, State::ChunkSizeEnd, chunk_size_end);
            }
        }
    }

    #[inline]
    fn chunk_size_end(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            if get_all28!(self) == 0 {
                callback_transition_fast!(self, context,
                                          on_chunk_size, get_all28!(self),
                                          State::Newline2, newline2);
            }

            callback_transition_fast!(self, context,
                                      on_chunk_size, get_all28!(self),
                                      State::ChunkSizeNewline, chunk_size_newline);
        } else if context.byte == b';' {
            callback_transition_fast!(self, context,
                                      on_chunk_size, get_all28!(self),
                                      State::ChunkExtensionName, chunk_extension_name);
        }

        Err(ParserError::ChunkSize(context.byte))
    }

    #[inline]
    fn chunk_extension_name(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_tokens!(context, ParserError::ChunkExtensionName,
            // stop on these bytes
            context.byte == b'=',

            // on end-of-stream
            callback_eos_expr!(self, context, on_chunk_extension_name)
        );

        callback_ignore_transition_fast!(self, context,
                                         on_chunk_extension_name,
                                         State::ChunkExtensionValue, chunk_extension_value);
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
                                                 State::ChunkSizeNewline, chunk_size_newline);
            },
            b';' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_chunk_extension_value,
                                                 State::ChunkExtensionName, chunk_extension_name);
            },
            _ => {
                transition_fast!(self, context, State::ChunkExtensionQuotedValue,
                                 chunk_extension_quoted_value);
            }
        }
    }

    #[inline]
    fn chunk_extension_quoted_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_quoted_value!(self, context, ParserError::ChunkExtensionValue,
                              on_chunk_extension_value);

        if context.byte == b'"' {
            callback_ignore_transition_fast!(self, context,
                                             on_chunk_extension_value,
                                             State::ChunkExtensionSemiColon,
                                             chunk_extension_semi_colon);
        } else {
            callback_ignore_transition_fast!(self, context,
                                             on_chunk_extension_value,
                                             State::ChunkExtensionEscapedValue,
                                             chunk_extension_escaped_value);
        }
    }

    #[inline]
    fn chunk_extension_escaped_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_visible_7bit!(context.byte) || context.byte == b' ' {
            callback_transition_fast!(self, context,
                                      on_chunk_extension_value, &[context.byte],
                                      State::ChunkExtensionQuotedValue,
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
            transition!(self, context, State::ChunkExtensionName, chunk_extension_name);
        } else if context.byte == b'\r' {
            transition!(self, context, State::ChunkSizeNewline, chunk_size_newline);
        }

        Err(ParserError::ChunkExtensionName(context.byte))
    }

    #[inline]
    fn chunk_size_newline(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition!(self, context, State::ChunkData, chunk_data);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn chunk_data(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        /*
        if bs_available!(context) as u64 >= get_upper40!(self) {
            bs_collect_length!(context, get_upper40!(self) as usize);

            set_upper40!(self, 0);

            callback_transition!(self, context,
                                 on_chunk_data,
                                 State::ChunkDataNewline1, chunk_data_newline1);
        }

        bs_collect_length!(context, bs_available!(context));

        set_upper40!(self, get_upper40!(self) - bs_available!(context) as u64);
        */
        if collect_content_length!(self, context) {
            callback_transition!(self, context,
                                 on_chunk_data,
                                 State::ChunkDataNewline1, chunk_data_newline1);
        }

        callback_transition!(self, context,
                             on_chunk_data,
                             State::ChunkData, chunk_data);
    }

    #[inline]
    fn chunk_data_newline1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, State::ChunkDataNewline2, chunk_data_newline2);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn chunk_data_newline2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition_fast!(self, context, State::ChunkSize1, chunk_size1);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn multipart_hyphen1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'-' {
            transition_fast!(self, context, State::MultipartHyphen2, multipart_hyphen2);
        }

        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    fn multipart_hyphen2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'-' {
            transition_fast!(self, context, State::MultipartTryBoundary, multipart_try_boundary);
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
            || context.byte == b'+'
            || context.byte == b'\r',

            // on end-of-stream
            callback_eos_expr!(self, context, on_url_encoded_field)
        );

        match context.byte {
            b'=' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_field,
                                                 State::UrlEncodedValue, url_encoded_value);
            },
            b'%' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_field,
                                                 State::UrlEncodedFieldHex, url_encoded_field_hex);
            },
            b'&' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_field,
                                                 State::UrlEncodedFieldAmpersand,
                                                 url_encoded_field_ampersand);
            },
            b'+' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_field,
                                                 State::UrlEncodedFieldPlus,
                                                 url_encoded_field_plus);
            },
            _ => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_field,
                                                 State::FinishedNewline2, finished_newline2);
            }
        }
    }

    #[inline]
    fn url_encoded_field_ampersand(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        // param field without a value, so send an empty array
        callback_transition!(self, context,
                             on_url_encoded_value, b"",
                             State::UrlEncodedField, url_encoded_field);
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
                                         State::UrlEncodedField, url_encoded_field);
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
                             State::UrlEncodedField, url_encoded_field);
    }

    #[inline]
    fn url_encoded_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_visible!(context, ParserError::UrlEncodedValue,
            // stop on these bytes
               context.byte == b'%'
            || context.byte == b'&'
            || context.byte == b'+'
            || context.byte == b'\r'
            || context.byte == b'=',

            // on end-of-stream
            callback_eos_expr!(self, context, on_url_encoded_value)
        );

        match context.byte {
            b'%' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_value,
                                                 State::UrlEncodedValueHex, url_encoded_value_hex);
            },
            b'&' => {
                callback_ignore_transition!(self, context,
                                            on_url_encoded_value,
                                            State::UrlEncodedField,
                                            url_encoded_field);
            },
            b'+' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_value,
                                                 State::UrlEncodedValuePlus,
                                                 url_encoded_value_plus);
            },
            b'\r' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_value,
                                                 State::FinishedNewline2, finished_newline2);
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
                                         State::UrlEncodedValue, url_encoded_value);
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
                             State::UrlEncodedValue, url_encoded_value);
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
    fn finished_newline1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, State::FinishedNewline2, finished_newline2);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn finished_newline2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition_fast!(self, context, State::Finished, finished);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn finished(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_finished!(self, context);
    }
}
