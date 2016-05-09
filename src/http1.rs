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

//! Zero-copy streaming HTTP parser.

use super::Success;
use byte::hex_to_byte;
use byte::is_token;
use url::ParamHandler;

use std::fmt;

// State flag mask.
const FLAG_MASK: u64 = 0x7F;

// State flag shift.
const FLAG_SHIFT: u8 = 40;

// Length mask.
const LENGTH_MASK: u64 = 0xFFFFFFFFFF;

// Major HTTP version shift.
const MAJOR_SHIFT: u8 = 4;

// Minor HTTP version mask.
const MINOR_MASK: u64 = 0xF;

// Top 16 bits shift.
const TOP16_SHIFT: u8 = 48;

// Parser type mask.
const TYPE_MASK: u64 = 0x1;

// Parser type shift.
const TYPE_SHIFT: u8 = 41;

// Maximum chunk extension byte count to process before returning
// `ParserError::MaxChunkExtensionLength`.
const CFG_MAX_CHUNK_EXTENSION_LENGTH: u8 = 255;

// Maximum multipart boundary byte count to process before returning
// `ParserError::MaxMultipartBoundaryLength`.
const CFG_MAX_MULTIPART_BOUNDARY_LENGTH: u8 = 70;

// -------------------------------------------------------------------------------------------------

/// Invalid chunk extension.
pub const ERR_CHUNK_EXTENSION: &'static str = "Invalid chunk extension";

/// Invalid chunk size.
pub const ERR_CHUNK_SIZE: &'static str = "Invalid chunk size";

/// Invalid CRLF sequence.
pub const ERR_CRLF_SEQUENCE: &'static str = "Invalid CRLF sequence";

/// Last `Parser::parse()` call returned an `Error` and cannot continue.
pub const ERR_DEAD: &'static str = "Parser is dead";

/// Invalid header field.
pub const ERR_HEADER_FIELD: &'static str = "Invalid header field";

/// Invalid header value.
pub const ERR_HEADER_VALUE: &'static str = "Invalid header byte";

/// Invalid hex sequence.
pub const ERR_HEX_SEQUENCE: &'static str = "Invalid hex byte";

/// Maximum chunk extension length has been met.
pub const ERR_MAX_CHUNK_EXTENSION_LENGTH: &'static str = "Maximum chunk extension length";

/// Maximum multipart boundary length.
pub const ERR_MAX_MULTIPART_BOUNDARY_LENGTH: &'static str = "Maximum multipart boundary length";

/// Invalid method.
pub const ERR_METHOD: &'static str = "Invalid method";

/// Missing content length header.
pub const ERR_MISSING_CONTENT_LENGTH: &'static str = "Missing Content-Length header";

/// Invalid multipart boundary.
pub const ERR_MULTIPART_BOUNDARY: &'static str = "Invalid multipart boundary";

/// Invalid status.
pub const ERR_STATUS: &'static str = "Invalid status";

/// Invalid status code.
pub const ERR_STATUS_CODE: &'static str = "Invalid status code";

/// Invalid URL.
pub const ERR_URL: &'static str = "Invalid URL";

/// Invalid URL encoded field.
pub const ERR_URL_ENCODED_FIELD: &'static str = "Invalid URL encoded field";

/// Invalid URL encoded value.
pub const ERR_URL_ENCODED_VALUE: &'static str = "Invalid URL encoded value";

/// Invalid version.
pub const ERR_VERSION: &'static str = "Invalid HTTP version";

// -------------------------------------------------------------------------------------------------

// Flags used to track state details.
bitflags! {
    flags Flag: u64 {
        // Parsing chunked transfer encoding.
        const F_CHUNKED = 1 << 0,

        // Parsing data that needs to check against content length.
        const F_CONTENT_LENGTH = 1 << 1,

        // Parsing multipart data.
        const F_MULTIPART = 1 << 2
    }
}

// -------------------------------------------------------------------------------------------------

/// Connection.
#[derive(Clone,Copy,PartialEq)]
#[repr(u8)]
pub enum Connection {
    /// No connection.
    None,

    /// Close connection.
    Close,

    /// Keep connection alive.
    KeepAlive,

    /// Upgrade connection.
    Upgrade
}

impl fmt::Display for Connection {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Connection::None => {
                write!(formatter, "Connection::None")
            },
            Connection::Close => {
                write!(formatter, "Connection::Close")
            },
            Connection::KeepAlive => {
                write!(formatter, "Connection::KeepAlive")
            },
            Connection::Upgrade => {
                write!(formatter, "Connection::Upgrade")
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Content length.
#[derive(Clone,Copy,PartialEq)]
pub enum ContentLength {
    /// No content length.
    None,

    /// Specified content length.
    Specified(u64)
}

impl fmt::Display for ContentLength {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ContentLength::None => {
                write!(formatter, "ContentLength::None")
            },
            ContentLength::Specified(x) => {
                write!(formatter, "ContentLength::Specified({})", x)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Content type.
#[derive(Clone,PartialEq)]
pub enum ContentType {
    /// No content type.
    None,

    /// Multipart content type.
    Multipart(Vec<u8>),

    /// URL encoded content type.
    UrlEncoded,

    /// Other content type.
    Other(Vec<u8>),
}

impl fmt::Display for ContentType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ContentType::None => {
                write!(formatter, "ContentType::None")
            },
            ContentType::Multipart(ref x) => {
                write!(formatter, "ContentType::Multipart({:?})", x)
            },
            ContentType::UrlEncoded => {
                write!(formatter, "ContentType::UrlEncoded")
            },
            ContentType::Other(ref x) => {
                write!(formatter, "ContentType::Other({:?})", x)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Parser error messages.
#[derive(Clone,Copy,PartialEq)]
pub enum ParserError {
    /// Invalid chunk extension.
    ChunkExtension(&'static str, u8),

    /// Invalid chunk size.
    ChunkSize(&'static str, u8),

    /// Invalid CRLF sequence.
    CrlfSequence(&'static str, u8),

    /// Parsing has failed, but `Parser::parse()` is executed again.
    Dead(&'static str),

    /// Invalid header field.
    HeaderField(&'static str, u8),

    /// Invalid header value.
    HeaderValue(&'static str, u8),

    /// Maximum chunk extension length has been met.
    MaxChunkExtensionLength(&'static str, u8),

    /// Maximum content length has been met.
    MaxContentLength(&'static str, u8),

    /// Missing an expected Content-Length header.
    MissingContentLength(&'static str),

    /// Invalid request method.
    Method(&'static str, u8),

    /// Invalid multipart boundary.
    MultipartBoundary(&'static str, u8),

    /// Invalid status.
    Status(&'static str, u8),

    /// Invalid status code.
    StatusCode(&'static str, u8),

    /// Invalid URL character.
    Url(&'static str, u8),

    /// Invalid URL encoded field.
    UrlEncodedField(&'static str, u8),

    /// Invalid URL encoded value.
    UrlEncodedValue(&'static str, u8),

    /// Invalid HTTP version.
    Version(&'static str, u8),
}

/// Parser return values.
pub enum ParserValue {
    /// Continue the parser loop.
    Continue,

    /// Exit the parser loop.
    Exit(Success),

    /// Shift the stream slice over by a specified length, and continue the parser loop.
    ShiftStream(usize)
}

/// Indicates the current parser state.
///
/// These states are in the order that they are processed.
#[derive(Clone,Copy,Debug,PartialEq)]
#[repr(u8)]
pub enum State {
    /// An error was returned from a call to `Parser::parse()`.
    Dead = 1,

    /// Parsing has finished successfully.
    Finished,

    // ---------------------------------------------------------------------------------------------
    // REQUEST
    // ---------------------------------------------------------------------------------------------

    /// Stripping linear white space before method.
    StripRequestMethod,

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

    /// Stripping linear white space before response HTTP version.
    StripResponseHttp,

    /// Parsing response HTTP version byte 1.
    ResponseHttp1,

    /// Parsing response HTTP version byte 2.
    ResponseHttp2,

    /// Parsing response HTTP version byte 3.
    ResponseHttp3,

    /// Parsing response HTTP version byte 4.
    ResponseHttp4,

    /// Parsing response HTTP version byte 5.
    ResponseHttp5,

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

    /// Parsing header quoted value.
    QuotedHeaderValue,

    /// Parsing header quoted escaped value.
    QuotedEscapedHeaderValue,

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

    /// Parsing body.
    Body,

    /// Unparsable content.
    Content,

    /// Parsing chunk size byte 1.
    ChunkSize1,

    /// Parsing chunk size byte 2.
    ChunkSize2,

    /// Parsing chunk size byte 3.
    ChunkSize3,

    /// Parsing chunk size byte 4.
    ChunkSize4,

    /// Parsing chunk size byte 5.
    ChunkSize5,

    /// Parsing chunk size byte 6.
    ChunkSize6,

    /// Parsing chunk size byte 7.
    ChunkSize7,

    /// Parsing chunk size byte 8.
    ChunkSize8,

    /// Parsing chunk size byte 9.
    ChunkSize9,

    /// Parsing chunk size byte 10.
    ChunkSize10,

    /// Parsing chunk extension.
    ChunkExtension,

    /// Parsing line feed after chunk size.
    ChunkSizeNewline,

    /// Parsing chunk data.
    ChunkData,

    /// Parsing carriage return after chunk data.
    ChunkDataNewline1,

    /// Parsing line feed after chunk data.
    ChunkDataNewline2,

    /// Parsing first hyphen before and after multipart boundary.
    MultipartHyphen1,

    /// Parsing second hyphen before and after multipart boundary.
    MultipartHyphen2,

    /// Parsing multipart boundary.
    MultipartBoundary,

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

    /// Parsing carriage return after URL encoded data.
    UrlEncodedNewline1,

    /// Parsing line feed after URL encoded data.
    UrlEncodedNewline2
}

/// Transfer encoding.
#[derive(Clone,PartialEq)]
pub enum TransferEncoding {
    /// No transfer encoding.
    None,

    /// Chunked transfer encoding.
    Chunked,

    /// Compress transfer encoding.
    Compress,

    /// Deflate transfer encoding.
    Deflate,

    /// Gzip transfer encoding.
    Gzip,

    /// Other transfer encoding.
    Other(Vec<u8>)
}

impl fmt::Display for TransferEncoding {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TransferEncoding::None => {
                write!(formatter, "TransferEncoding::None")
            },
            TransferEncoding::Chunked => {
                write!(formatter, "TransferEncoding::Chunked")
            },
            TransferEncoding::Compress => {
                write!(formatter, "TransferEncoding::Compress")
            },
            TransferEncoding::Deflate => {
                write!(formatter, "TransferEncoding::Deflate")
            },
            TransferEncoding::Gzip => {
                write!(formatter, "TransferEncoding::Gzip")
            },
            TransferEncoding::Other(ref x) => {
                write!(formatter, "TransferEncoding::Other({:?})", x)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Parser state function type.
pub type StateFunction<T> = fn(&mut Parser<T>, &mut ParserContext<T>)
    -> Result<ParserValue, ParserError>;

// -------------------------------------------------------------------------------------------------
// BIT DATA MACROS
// -------------------------------------------------------------------------------------------------

// Retrieve the length bits.
macro_rules! get_length {
    ($parser:expr) => (
        $parser.bit_data & LENGTH_MASK
    );
}

// Retrieve the top 16 bits.
macro_rules! get_top16 {
    ($parser:expr) => (
        ($parser.bit_data >> TOP16_SHIFT)
    );
}

// Indicates that a state flag bit.
macro_rules! has_flag {
    ($parser:expr, $flag:expr) => ({
        (($parser.bit_data >> FLAG_SHIFT) & FLAG_MASK) & $flag.bits == $flag.bits
    });
}

// Increment length bits.
macro_rules! inc_length {
    ($parser:expr, $length:expr) => ({
        $parser.bit_data |= $length & LENGTH_MASK;
    });
}

// Reset length bits.
macro_rules! reset_length {
    ($parser:expr) => ({
        $parser.bit_data &= !LENGTH_MASK;
    });
}

// Set a state flag bit.
macro_rules! set_flag {
    ($parser:expr, $flag:expr) => ({
        $parser.bit_data |= ($flag.bits & FLAG_MASK) << FLAG_SHIFT;
    });
}

// Set length bits.
macro_rules! set_length {
    ($parser:expr, $length:expr) => ({
        $parser.bit_data |= $length & LENGTH_MASK;
    });
}

// Set the top 16 bits.
macro_rules! set_top16 {
    ($parser:expr, $bits:expr) => ({
        $parser.bit_data |= $bits << TOP16_SHIFT;
    });
}

// Unset a state flag bit.
macro_rules! unset_flag {
    ($parser:expr, $flag:expr) => ({
        $parser.bit_data &= !(($flag.bits & FLAG_MASK) << FLAG_SHIFT);
    });
}

// -------------------------------------------------------------------------------------------------
// STREAM MACROS
// -------------------------------------------------------------------------------------------------

// Execute a callback and if it returns true, execute a block, otherwise exit with callback status.
macro_rules! callback {
    ($parser:expr, $context:expr, $function:ident, $block:block) => ({
        let slice = collected_bytes!($context);

        if slice.len() > 0 {
            if $context.handler.$function(slice) {
                $block
            } else {
                exit_callback!($parser, $context);
            }
        } else {
            $block
        }
    });
}

// Execute a callback with specified data, and if it returns true, execute a block, otherwise exit
// with callback status.
macro_rules! callback_data {
    ($parser:expr, $context:expr, $data:expr, $function:ident, $block:block) => ({
        if $context.handler.$function($data) {
            $block
        } else {
            exit_callback!($parser, $context);
        }
    });
}

// Execute a callback ignoring the last marked byte, and if it returns true, execute a block,
// otherwise exit with callback status.
macro_rules! callback_ignore {
    ($parser:expr, $context:expr, $function:ident, $block:block) => ({
        let slice = &$context.stream[$context.mark_index..$context.stream_index - 1];

        if slice.len() > 0 {
            if $context.handler.$function(slice) {
                $block
            } else {
                exit_callback!($parser, $context);
            }
        } else {
            $block
        }
    });
}

// Execute a callback and if it returns true, exit with eof status, otherwise exit with callback
// status.
macro_rules! callback_or_eof {
    ($parser:expr, $context:expr, $function:ident) => (
        callback!($parser, $context, $function, {
            exit_eof!($parser, $context);
        });
    );
}

// Change parser state.
macro_rules! change_state {
    ($parser:expr, $context:expr) => ({
        $context.mark_index = $context.stream_index;

        return Ok(ParserValue::Continue);
    });
}

// Change parser state fast, without returning immediately.
macro_rules! change_state_fast {
    ($parser:expr, $context:expr) => ({
        $context.mark_index = $context.stream_index;

        return ($parser.state_function)($parser, $context);
    });
}

// Retrieve a slice of collected bytes.
macro_rules! collected_bytes {
    ($context:expr) => (
        &$context.stream[$context.mark_index..$context.stream_index]
    );
}

// Collect digits as a single numerical value.
macro_rules! collect_digits {
    ($parser:expr, $context:expr, $digit:expr, $max:expr, $error:expr, $error_msg:expr,
     $eof_block:block) => ({
        loop {
            if is_eof!($context) {
                $eof_block
            }

            next!($context);

            if is_digit!($context.byte) {
                $digit *= 10;
                $digit += ($context.byte - b'0') as u16;

                if $digit > $max {
                    exit_error!($parser, $context, $error($error_msg, $context.byte));
                }
            } else {
                break;
            }
        }
    });
}

// Collect all 7-bit non-control bytes.
macro_rules! collect_safe {
    ($parser:expr, $context:expr, $stop1:expr, $stop2:expr, $error:expr, $error_msg:expr,
     $eof_block:block) => ({
        loop {
            if is_eof!($context) {
                $eof_block
            }

            next!($context);

            if $stop1 == $context.byte || $stop2 == $context.byte {
                break;
            } else if is_control!($context.byte) || !is_ascii!($context.byte) {
                exit_error!($parser, $context, $error($error_msg, $context.byte));
            }
        }
    });

    ($parser:expr, $context:expr, $stop:expr, $error:expr, $error_msg:expr,
     $eof_block:block) => ({
        loop {
            if is_eof!($context) {
                $eof_block
            }

            next!($context);

            if $stop == $context.byte {
                break;
            } else if is_control!($context.byte) || !is_ascii!($context.byte) {
                exit_error!($parser, $context, $error($error_msg, $context.byte));
            }
        }
    });
}

// Collect tokens.
macro_rules! collect_tokens {
    ($parser:expr, $context:expr, $stop:expr, $error:expr, $error_msg:expr,
     $eof_block:block) => ({
        loop {
            if is_eof!($context) {
                $eof_block
            }

            next!($context);

            if $stop == $context.byte {
                break;
            } else if !is_token($context.byte) {
                exit_error!($parser, $context, $error($error_msg, $context.byte));
            }
        }
    });
}

// Collect tokens up until a certain limit.
//
// Use the top 16 bits to store the count of processed bytes, limiting $limit to 65536.
macro_rules! collect_tokens_limit {
    ($parser:expr, $context:expr, $stop:expr, $limit:expr, $error:expr, $error_msg:expr,
     $limit_error:expr, $limit_error_msg:expr, $eof_block:block) => ({
        loop {
            if is_eof!($context) {
                $eof_block
            }

            if get_top16!($parser) == $limit as u16 {
                exit_error!($parser, $context, $limit_error($limit_error_msg, $context.byte));
            }

            set_top16!($parser, get_top16!($parser) + 1);

            next!($context);

            if $stop == $context.byte {
                break;
            } else if !is_token($context.byte) {
                exit_error!($parser, $context, $error($error_msg, $context.byte));
            }
        }
    });
}

// Consume spaces and tabs.
macro_rules! consume_space_tab {
    ($parser:expr, $context:expr) => ({
        loop {
            if is_eof!($context) {
                exit_eof!($parser, $context);
            }

            next!($context);

            if $context.byte != b' ' && $context.byte != b'\t' {
                break;
            }
        }
    });
}

// Exit parser function with a callback status.
macro_rules! exit_callback {
    ($parser:expr, $context:expr) => ({
        $parser.byte_count += $context.stream_index;

        return Ok(ParserValue::Exit(Success::Callback($context.stream_index)));
    });
}

// Exit parser function with an EOF status.
macro_rules! exit_eof {
    ($parser:expr, $context:expr) => ({
        $parser.byte_count += $context.stream_index;

        return Ok(ParserValue::Exit(Success::Eof($context.stream_index)));
    });
}

// Exit parser function with an error.
macro_rules! exit_error {
    ($parser:expr, $context:expr, $error:expr) => ({
        $parser.byte_count += $context.stream_index;
        $parser.state       = State::Dead;

        return Err($error);
    });
}

// Exit parser with finished status.
macro_rules! exit_finished {
    ($parser:expr, $context:expr) => ({
        $parser.byte_count += $context.stream_index;
        $parser.state       = State::Finished;

        return Ok(ParserValue::Exit(Success::Finished($context.stream_index)));
    });
}

// Exit parser function with an EOF status if the stream is EOF, otherwise do nothing.
macro_rules! exit_if_eof {
    ($parser:expr, $context:expr) => (
        if is_eof!($context) {
            exit_eof!($parser, $context);
        }
    );
}

// Indicates that a specified amount of bytes are available.
macro_rules! has_bytes {
    ($context:expr, $length:expr) => (
        $context.stream_index + $length <= $context.stream.len()
    );
}

// Indicates that we're at the end of the stream.
macro_rules! is_eof {
    ($context:expr) => (
        $context.stream_index == $context.stream.len()
    );
}

// Jump a specified amount of bytes.
macro_rules! jump_bytes {
    ($context:expr, $length:expr) => ({
        $context.stream_index += $length;
    });
}

// Advance the stream one byte.
macro_rules! next {
    ($context:expr) => ({
        $context.stream_index += 1;
        $context.byte   = $context.stream[$context.stream_index - 1]
    });
}

// Peek at a slice of available bytes.
macro_rules! peek_bytes {
    ($context:expr, $length:expr) => (
        &$context.stream[$context.stream_index..$context.stream_index + $length]
    );
}

// Replay the most recent byte by rewinding the stream index 1 byte.
macro_rules! replay {
    ($context:expr) => (
        $context.stream_index -= 1;
    );
}

// Set state and state function.
macro_rules! set_state {
    ($parser:expr, $state:expr, $state_function:ident) => ({
        $parser.state          = $state;
        $parser.state_function = Parser::$state_function;
    });
}

// -------------------------------------------------------------------------------------------------

/// Type that handles HTTP parser events.
#[allow(unused_variables)]
pub trait HttpHandler {
    /// Retrieve the most recent Connection header.
    fn get_connection(&mut self) -> Connection {
        Connection::None
    }

    /// Retrieve the most recent Content-Length header.
    fn get_content_length(&mut self) -> ContentLength {
        ContentLength::None
    }

    /// Retrieve the most recent Content-Type header.
    fn get_content_type(&mut self) -> ContentType {
        ContentType::None
    }

    /// Retrieve the most recent Transfer-Encoding header.
    fn get_transfer_encoding(&mut self) -> TransferEncoding {
        TransferEncoding::None
    }

    /// Callback that is executed when raw body data has been received.
    ///
    /// This may be executed multiple times in order to supply the entire body.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_body(&mut self, body: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a chunk of data has been parsed.
    ///
    /// This may be executed multiple times in order to supply the entire chunk.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_chunk_data(&mut self, data: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a chunk extension has been located.
    ///
    /// This may be executed multiple times in order to supply the entire chunk extension.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_chunk_extension(&mut self, extension: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a chunk size has been located.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_chunk_size(&mut self, size: u64) -> bool {
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
    fn on_headers_finished(&mut self) -> bool {
        true
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

    /// Callback that is executed when the HTTP major version has been located.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_version(&mut self, major: u16, minor: u16) -> bool {
        true
    }
}

// -------------------------------------------------------------------------------------------------

/// Parser context data.
pub struct ParserContext<'a, T: HttpHandler + ParamHandler + 'a> {
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

impl<'a, T: HttpHandler + ParamHandler + 'a> ParserContext<'a, T> {
    /// Create a new `ParserContext`.
    pub fn new(handler: &'a mut T, stream: &'a [u8]) -> ParserContext<'a, T> {
        ParserContext{ byte:         0,
                       handler:      handler,
                       mark_index:   0,
                       stream:       stream,
                       stream_index: 0 }
    }
}

/// Parser data.
pub struct Parser<T: HttpHandler + ParamHandler> {
    // Bit data that stores parser bit details.
    //
    // Bits 1-40: Used to store content length, chunk size, and HTTP major/minor versions.
    //            Allows value up to 1,099,511,627,775 bytes, which is more than adequate.
    //
    // Bit 41: If flagged, parser is parsing a request, otherwise it's parsing a response.
    //
    // Bits 42-48: State flags that are checked when states have a dual purpose, such as when
    //             header parsing is also used to parse chunked data trailers.
    //
    // Bits 49-64: For the most part unused, except when parsing chunk extension this stores
    //             the amount of parsed bytes, limiting chunk extensions to 255 bytes.
    bit_data: u64,

    // Total byte count processed for headers, and body.
    // Once the headers are finished processing, this is reset to 0 to track the body length.
    byte_count: usize,

    // Content type.
    content_type: ContentType,

    // Current state.
    state: State,

    // Current state function.
    state_function: StateFunction<T>
}

// Chunk size macro.
macro_rules! chunk_size {
    ($parser:expr, $context:expr, $state:expr, $state_function:ident) => ({
        exit_if_eof!($parser, $context);
        next!($context);

        match hex_to_byte(&[$context.byte]) {
            Some(byte) => {
                set_length!($parser, get_length!($parser) << 4);
                inc_length!($parser, byte as u64);

                set_state!($parser, $state, $state_function);
                change_state_fast!($parser, $context);
            },
            None => {
                if get_length!($parser) == 0 {
                    set_state!($parser, State::Newline2, newline2);

                    callback_data!($parser, $context, get_length!($parser), on_chunk_size, {
                        change_state!($parser, $context);
                    });
                } else if $context.byte == b'\r' {
                    set_state!($parser, State::ChunkSizeNewline, chunk_size_newline);

                    callback_data!($parser, $context, get_length!($parser), on_chunk_size, {
                        change_state!($parser, $context);
                    });
                } else if $context.byte == b';' {
                    set_state!($parser, State::ChunkExtension, chunk_extension);

                    callback_data!($parser, $context, get_length!($parser), on_chunk_size, {
                        change_state!($parser, $context);
                    });
                } else {
                    exit_error!($parser, $context, ParserError::ChunkSize(ERR_CHUNK_SIZE,
                                                                          $context.byte));
                }
            }
        }
    });
}

// Retrieve the length bits of bit_data.
macro_rules! get_length {
    ($parser:expr) => (
        $parser.bit_data & LENGTH_MASK
    );
}

// Retrieve the top 16 bits of bit_data.
macro_rules! get_top16 {
    ($parser:expr) => (
        ($parser.bit_data >> TOP16_SHIFT)
    );
}

// Indicates that a state flag bit is set within bit_data.
macro_rules! has_flag {
    ($parser:expr, $flag:expr) => ({
        (($parser.bit_data >> FLAG_SHIFT) & FLAG_MASK) & $flag.bits == $flag.bits
    });
}

// Increment length bits of bit_data.
macro_rules! inc_length {
    ($parser:expr, $length:expr) => ({
        $parser.bit_data |= $length & LENGTH_MASK;
    });
}

// Reset length bits of bit_data.
macro_rules! reset_length {
    ($parser:expr) => ({
        $parser.bit_data &= !LENGTH_MASK;
    });
}

// Set a state flag bit in bit_data.
macro_rules! set_flag {
    ($parser:expr, $flag:expr) => ({
        $parser.bit_data |= ($flag.bits & FLAG_MASK) << FLAG_SHIFT;
    });
}

// Set length bits in bit_data.
macro_rules! set_length {
    ($parser:expr, $length:expr) => ({
        $parser.bit_data |= $length & LENGTH_MASK;
    });
}

// Set the top 16 bits of bit_data.
macro_rules! set_top16 {
    ($parser:expr, $bits:expr) => ({
        $parser.bit_data |= $bits << TOP16_SHIFT;
    });
}

// Unset a state flag bit in bit_data.
macro_rules! unset_flag {
    ($parser:expr, $flag:expr) => ({
        $parser.bit_data &= !(($flag.bits & FLAG_MASK) << FLAG_SHIFT);
    });
}

impl<T: HttpHandler + ParamHandler> Parser<T> {
    /// Create a new `Parser`.
    pub fn new(state: State, state_function: StateFunction<T>) -> Parser<T> {
        Parser{ bit_data:       if state == State::StripRequestMethod {
                                    1 << TYPE_SHIFT
                                } else {
                                    0
                                },
                byte_count:     0,
                content_type:   ContentType::None,
                state:          state,
                state_function: state_function }
    }

    /// Create a new `Parser` for request parsing.
    pub fn new_request() -> Parser<T> {
        Parser::new(State::StripRequestMethod, Parser::strip_request_method)
    }

    /// Create a new `Parser` for response parsing.
    pub fn new_response() -> Parser<T> {
        Parser::new(State::StripResponseHttp, Parser::strip_response_http)
    }

    /// Retrieve the processed byte count since the start of the message.
    pub fn get_byte_count(&self) -> usize {
        self.byte_count
    }

    /// Retrieve the state.
    pub fn get_state(&self) -> State {
        self.state
    }

    /// Retrieve the state function.
    pub fn get_state_function(&self) -> StateFunction<T> {
        self.state_function
    }

    /// Parse HTTP data.
    ///
    /// If `Success` is returned, you may resuming parsing data with an additional call to
    /// `Parser::parse()`. If `ParserError` is returned, parsing cannot be resumed without a call
    /// to `Parser::reset()`.
    #[inline]
    pub fn parse(&mut self, handler: &mut T, mut stream: &[u8]) -> Result<Success, ParserError> {
        let mut context = ParserContext::new(handler, stream);

        loop {
            match (self.state_function)(self, &mut context) {
                Ok(ParserValue::Continue) => {
                },
                Ok(ParserValue::ShiftStream(length)) => {
                    stream = &stream[length..];
                },
                Ok(ParserValue::Exit(success)) => {
                    return Ok(success);
                },
                Err(error) => {
                    return Err(error);
                }
            }
        }
    }

    /// Reset the `Parser` back to its original state.
    pub fn reset(&mut self) {
        self.byte_count   = 0;
        self.content_type = ContentType::None;

        if (self.bit_data >> TYPE_SHIFT) & TYPE_MASK == 1 {
            self.bit_data       = 1 << TYPE_SHIFT;
            self.state          = State::StripRequestMethod;
            self.state_function = Parser::strip_request_method;
        } else {
            self.bit_data       = 0;
            self.state          = State::StripResponseHttp;
            self.state_function = Parser::strip_response_http;
        }
    }

    // ---------------------------------------------------------------------------------------------
    // HEADER STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    pub fn pre_headers1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'\n' {
            set_state!(self, State::PreHeaders2, pre_headers2);
            change_state_fast!(self, context);
        } else {
            exit_error!(self, context, ParserError::CrlfSequence(ERR_CRLF_SEQUENCE, context.byte));
        }
    }

    #[inline]
    pub fn pre_headers2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'\r' {
            set_state!(self, State::Newline4, newline4);
            change_state_fast!(self, context);
        } else {
            replay!(context);
            set_state!(self, State::StripHeaderField, strip_header_field);
            change_state_fast!(self, context);
        }
    }

    #[inline]
    pub fn strip_header_field(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_space_tab!(self, context);
        replay!(context);
        set_state!(self, State::FirstHeaderField, first_header_field);
        change_state_fast!(self, context);
    }

    #[inline]
    #[cfg_attr(test, allow(cyclomatic_complexity))]
    pub fn first_header_field(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        macro_rules! field {
            ($header:expr, $length:expr) => ({
                jump_bytes!(context, $length);
                set_state!(self, State::StripHeaderValue, strip_header_value);
                callback_data!(self, context, $header, on_header_field, {
                    change_state_fast!(self, context);
                });
            });
        }

        if has_bytes!(context, 26) {
            // have enough bytes to compare common header fields immediately, without collecting
            // individual tokens
            if context.byte == b'C' {
                if b"Connection:" == peek_bytes!(context, 11) {
                    field!(b"Connection", 11);
                } else if b"Content-Type:" == peek_bytes!(context, 13) {
                    field!(b"Content-Type", 13);
                } else if b"Content-Length:" == peek_bytes!(context, 15) {
                    field!(b"Content-Length", 15);
                } else if b"Cookie:" == peek_bytes!(context, 7) {
                    field!(b"Cookie", 7);
                } else if b"Cache-Control:" == peek_bytes!(context, 14) {
                    field!(b"Cache-Control", 14);
                } else if b"Content-Security-Policy:" == peek_bytes!(context, 24) {
                    field!(b"Content-Security-Policy", 24);
                }
            } else if context.byte == b'A' {
                if b"Accept:" == peek_bytes!(context, 7) {
                    field!(b"Accept", 7);
                } else if b"Accept-Charset:" == peek_bytes!(context, 15) {
                    field!(b"Accept-Charset", 15);
                } else if b"Accept-Encoding:" == peek_bytes!(context, 16) {
                    field!(b"Accept-Encoding", 16);
                } else if b"Accept-Language:" == peek_bytes!(context, 16) {
                    field!(b"Accept-Language", 16);
                } else if b"Authorization:" == peek_bytes!(context, 14) {
                    field!(b"Authorization", 14);
                }
            } else if context.byte == b'L' {
                if b"Location:" == peek_bytes!(context, 9) {
                    field!(b"Location", 9);
                } else if b"Last-Modified:" == peek_bytes!(context, 14) {
                    field!(b"Last-Modified", 14);
                }
            } else if context.byte == b'P' {
                if b"Pragma:" == peek_bytes!(context, 7) {
                    field!(b"Pragma", 7);
                }
            } else if context.byte == b'S' {
                if b"Set-Cookie:" == peek_bytes!(context, 11) {
                    field!(b"Set-Cookie", 11);
                }
            } else if context.byte == b'T' {
                if b"Transfer-Encoding:" == peek_bytes!(context, 18) {
                    field!(b"Transfer-Encoding", 18);
                }
            } else if context.byte == b'U' {
                if b"User-Agent:" == peek_bytes!(context, 11) {
                    field!(b"User-Agent", 11);
                } else if b"Upgrade:" == peek_bytes!(context, 8) {
                    field!(b"Upgrade", 8);
                }
            } else if context.byte == b'X' {
                if b"X-Powered-By:" == peek_bytes!(context, 13) {
                    field!(b"X-Powered-By", 13);
                } else if b"X-Forwarded-For:" == peek_bytes!(context, 16) {
                    field!(b"X-Forwarded-For", 16);
                } else if b"X-Forwarded-Host:" == peek_bytes!(context, 17) {
                    field!(b"X-Forwarded-Host", 17);
                } else if b"X-XSS-Protection:" == peek_bytes!(context, 17) {
                    field!(b"X-XSS-Protection", 17);
                } else if b"X-WebKit-CSP:" == peek_bytes!(context, 13) {
                    field!(b"X-WebKit-CSP", 13);
                } else if b"X-Content-Security-Policy:" == peek_bytes!(context, 26) {
                    field!(b"X-Content-Security-Policy", 26);
                }
            } else if context.byte == b'W' {
                if b"WWW-Authenticate:" == peek_bytes!(context, 17) {
                    field!(b"WWW-Authenticate", 17);
                }
            }
        }

        exit_if_eof!(self, context);
        set_state!(self, State::HeaderField, header_field);
        change_state_fast!(self, context);
    }

    #[inline]
    pub fn header_field(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_tokens!(self, context, b':', ParserError::HeaderField, ERR_HEADER_FIELD, {
            callback_or_eof!(self, context, on_header_field);
        });

        set_state!(self, State::StripHeaderValue, strip_header_value);
        callback_ignore!(self, context, on_header_field, {
            change_state_fast!(self, context);
        });
    }

    #[inline]
    pub fn strip_header_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_space_tab!(self, context);

        if context.byte == b'"' {
            set_state!(self, State::QuotedHeaderValue, quoted_header_value);
            change_state_fast!(self, context);
        }

        replay!(context);
        set_state!(self, State::HeaderValue, header_value);
        change_state_fast!(self, context);
    }

    #[inline]
    pub fn header_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_safe!(self, context, b'\r', ParserError::HeaderValue, ERR_HEADER_VALUE, {
            callback_or_eof!(self, context, on_header_value);
        });

        set_state!(self, State::Newline2, newline2);
        callback_ignore!(self, context, on_header_value, {
            change_state_fast!(self, context);
        });
    }

    #[inline]
    pub fn quoted_header_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_safe!(self, context, b'"', b'\\', ParserError::HeaderValue, ERR_HEADER_VALUE, {
            callback_or_eof!(self, context, on_header_value);
        });

        if context.byte == b'"' {
            set_state!(self, State::Newline1, newline1);
        } else {
            set_state!(self, State::QuotedEscapedHeaderValue, quoted_escaped_header_value);
        }

        callback_ignore!(self, context, on_header_value, {
            change_state!(self, context);
        });
    }

    #[inline]
    pub fn quoted_escaped_header_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);
        set_state!(self, State::QuotedHeaderValue, quoted_header_value);
        callback_data!(self, context, &[context.byte], on_header_value, {
            change_state!(self, context);
        });
    }

    #[inline]
    pub fn newline1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        if has_bytes!(context, 2) && b"\r\n" == peek_bytes!(context, 2) {
            set_state!(self, State::Newline3, newline3);
            change_state_fast!(self, context);
        }

        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'\r' {
            set_state!(self, State::Newline2, newline2);
            change_state_fast!(self, context);
        }

        exit_error!(self, context, ParserError::CrlfSequence(ERR_CRLF_SEQUENCE, context.byte));
    }

    #[inline]
    pub fn newline2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'\n' {
            set_state!(self, State::Newline3, newline3);
            change_state_fast!(self, context);
        }

        exit_error!(self, context, ParserError::CrlfSequence(ERR_CRLF_SEQUENCE, context.byte));
    }

    #[inline]
    pub fn newline3(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'\r' {
            set_state!(self, State::Newline4, newline4);
            change_state_fast!(self, context);
        } else if context.byte == b' ' || context.byte == b'\t' {
            set_state!(self, State::StripHeaderValue, strip_header_value);
            callback_data!(self, context, b" ", on_header_value, {
                change_state!(self, context);
            });
        } else {
            replay!(context);
            set_state!(self, State::StripHeaderField, strip_header_field);
            change_state!(self, context);
        }
    }

    #[inline]
    pub fn newline4(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'\n' {
            set_state!(self, State::Body, body);

            if context.handler.on_headers_finished() {
                if has_flag!(self, F_CHUNKED) {
                    exit_finished!(self, context);
                }

                change_state_fast!(self, context);
            } else if has_flag!(self, F_CHUNKED) {
                exit_finished!(self, context);
            } else {
                exit_callback!(self, context);
            }
        }

        exit_error!(self, context, ParserError::CrlfSequence(ERR_CRLF_SEQUENCE, context.byte));
    }

    // ---------------------------------------------------------------------------------------------
    // REQUEST STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    pub fn strip_request_method(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_space_tab!(self, context);
        replay!(context);
        set_state!(self, State::RequestMethod, request_method);
        change_state_fast!(self, context);
    }

    #[inline]
    pub fn request_method(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        macro_rules! method {
            ($method:expr, $length:expr) => (
                jump_bytes!(context, $length);
                set_state!(self, State::StripRequestUrl, strip_request_url);
                callback_data!(self, context, $method, on_method, {
                    change_state_fast!(self, context);
                });
            );
        }

        if has_bytes!(context, 8) {
            // have enough bytes to compare all known methods immediately, without collecting
            // individual tokens
            if b"GET " == peek_bytes!(context, 4) {
                method!(b"GET", 4);
            } else if b"POST " == peek_bytes!(context, 5) {
                method!(b"POST", 5);
            } else if b"PUT " == peek_bytes!(context, 4) {
                method!(b"PUT", 4);
            } else if b"DELETE " == peek_bytes!(context, 7) {
                method!(b"DELETE", 7);
            } else if b"CONNECT " == peek_bytes!(context, 8) {
                method!(b"CONNECT", 8);
            } else if b"OPTIONS " == peek_bytes!(context, 8) {
                method!(b"OPTIONS", 8);
            } else if b"HEAD " == peek_bytes!(context, 5) {
                method!(b"HEAD", 5);
            } else if b"TRACE " == peek_bytes!(context, 6) {
                method!(b"TRACE", 6);
            }
        }

        collect_tokens!(self, context, b' ', ParserError::Method, ERR_METHOD, {
            callback_or_eof!(self, context, on_method);
        });

        replay!(context);
        set_state!(self, State::StripRequestUrl, strip_request_url);
        callback!(self, context, on_method, {
            change_state_fast!(self, context);
        });
    }

    #[inline]
    pub fn strip_request_url(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_space_tab!(self, context);
        replay!(context);
        set_state!(self, State::RequestUrl, request_url);
        change_state_fast!(self, context);
    }

    #[inline]
    pub fn request_url(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_safe!(self, context, b' ', ParserError::Url, ERR_URL, {
            callback_or_eof!(self, context, on_url);
        });

        replay!(context);
        set_state!(self, State::StripRequestHttp, strip_request_http);
        callback!(self, context, on_url, {
            change_state_fast!(self, context);
        });
    }

    #[inline]
    pub fn strip_request_http(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_space_tab!(self, context);
        replay!(context);
        set_state!(self, State::RequestHttp1, request_http1);
        change_state_fast!(self, context);
    }

    #[inline]
    pub fn request_http1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        macro_rules! version {
            ($major:expr, $minor:expr, $length:expr) => (
                jump_bytes!(context, $length);
                set_state!(self, State::PreHeaders1, pre_headers1);

                if context.handler.on_version($major, $minor) {
                    change_state_fast!(self, context);
                } else {
                    exit_callback!(self, context);
                }
            );
        }

        if has_bytes!(context, 9) {
            // have enough bytes to compare all known versions immediately, without collecting
            // individual tokens
            if b"HTTP/1.1\r" == peek_bytes!(context, 9) {
                version!(1, 1, 9);
            } else if b"HTTP/2.0\r" == peek_bytes!(context, 9) {
                version!(2, 0, 9);
            } else if b"HTTP/1.0\r" == peek_bytes!(context, 9) {
                version!(1, 0, 9);
            }
        }

        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'H' || context.byte == b'h' {
            set_state!(self, State::RequestHttp2, request_http2);
            change_state_fast!(self, context);
        } else {
            exit_error!(self, context, ParserError::Version(ERR_VERSION, context.byte));
        }
    }

    #[inline]
    pub fn request_http2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'T' || context.byte == b't' {
            set_state!(self, State::RequestHttp3, request_http3);
            change_state_fast!(self, context);
        } else {
            exit_error!(self, context, ParserError::Version(ERR_VERSION, context.byte));
        }
    }

    #[inline]
    pub fn request_http3(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'T' || context.byte == b't' {
            set_state!(self, State::RequestHttp4, request_http4);
            change_state_fast!(self, context);
        } else {
            exit_error!(self, context, ParserError::Version(ERR_VERSION, context.byte));
        }
    }

    #[inline]
    pub fn request_http4(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'P' || context.byte == b'p' {
            set_state!(self, State::RequestHttp5, request_http5);
            change_state_fast!(self, context);
        } else {
            exit_error!(self, context, ParserError::Version(ERR_VERSION, context.byte));
        }
    }

    #[inline]
    pub fn request_http5(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'/' {
            reset_length!(self);

            set_state!(self, State::RequestVersionMajor, request_version_major);
            change_state_fast!(self, context);
        } else {
            exit_error!(self, context, ParserError::Version(ERR_VERSION, context.byte));
        }
    }

    #[inline]
    pub fn request_version_major(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = get_length!(self) as u16;

        collect_digits!(self, context, digit, 999, ParserError::Version, ERR_VERSION, {
            inc_length!(self, digit as u64);

            exit_eof!(self, context);
        });

        set_length!(self, (digit << MAJOR_SHIFT) as u64);

        if context.byte != b'.' {
            exit_error!(self, context, ParserError::Version(ERR_VERSION, context.byte));
        }

        set_state!(self, State::RequestVersionMinor, request_version_minor);
        change_state_fast!(self, context);
    }

    #[inline]
    pub fn request_version_minor(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit: u16 = (get_length!(self) & MINOR_MASK) as u16;

        collect_digits!(self, context, digit, 999, ParserError::Version, ERR_VERSION, {
            inc_length!(self, digit as u64);

            exit_eof!(self, context);
        });

        set_state!(self, State::PreHeaders1, pre_headers1);

        if context.handler.on_version((get_length!(self) >> MAJOR_SHIFT) as u16, digit) {
            change_state_fast!(self, context);
        } else {
            exit_callback!(self, context);
        }
    }

    // ---------------------------------------------------------------------------------------------
    // RESPONSE STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    pub fn strip_response_http(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_space_tab!(self, context);
        replay!(context);
        set_state!(self, State::ResponseHttp1, response_http1);
        change_state_fast!(self, context);
    }

    #[inline]
    pub fn response_http1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        macro_rules! version {
            ($major:expr, $minor:expr, $length:expr) => (
                jump_bytes!(context, $length);
                set_state!(self, State::StripResponseStatusCode, strip_response_status_code);

                if context.handler.on_version($major, $minor) {
                    change_state_fast!(self, context);
                } else {
                    exit_callback!(self, context);
                }
            );
        }

        if has_bytes!(context, 9) {
            // have enough bytes to compare all known versions immediately, without collecting
            // individual tokens
            if b"HTTP/1.1 " == peek_bytes!(context, 9) {
                version!(1, 1, 9);
            } else if b"HTTP/2.0 " == peek_bytes!(context, 9) {
                version!(2, 0, 9);
            } else if b"HTTP/1.0 " == peek_bytes!(context, 9) {
                version!(1, 0, 9);
            }
        }

        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'H' || context.byte == b'h' {
            set_state!(self, State::ResponseHttp2, response_http2);
            change_state_fast!(self, context);
        } else {
            exit_error!(self, context, ParserError::Version(ERR_VERSION, context.byte));
        }
    }

    #[inline]
    pub fn response_http2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'T' || context.byte == b't' {
            set_state!(self, State::ResponseHttp3, response_http3);
            change_state_fast!(self, context);
        } else {
            exit_error!(self, context, ParserError::Version(ERR_VERSION, context.byte));
        }
    }

    #[inline]
    pub fn response_http3(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'T' || context.byte == b't' {
            set_state!(self, State::ResponseHttp4, response_http4);
            change_state_fast!(self, context);
        } else {
            exit_error!(self, context, ParserError::Version(ERR_VERSION, context.byte));
        }
    }

    #[inline]
    pub fn response_http4(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'P' || context.byte == b'p' {
            set_state!(self, State::ResponseHttp5, response_http5);
            change_state_fast!(self, context);
        } else {
            exit_error!(self, context, ParserError::Version(ERR_VERSION, context.byte));
        }
    }

    #[inline]
    pub fn response_http5(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'/' {
            reset_length!(self);

            set_state!(self, State::ResponseVersionMajor, response_version_major);
            change_state_fast!(self, context);
        } else {
            exit_error!(self, context, ParserError::Version(ERR_VERSION, context.byte));
        }
    }

    #[inline]
    pub fn response_version_major(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = get_length!(self) as u16;

        collect_digits!(self, context, digit, 999, ParserError::Version, ERR_VERSION, {
            set_length!(self, digit as u64);

            exit_eof!(self, context);
        });

        set_length!(self, (digit << MAJOR_SHIFT) as u64);

        if context.byte != b'.' {
            exit_error!(self, context, ParserError::Version(ERR_VERSION, context.byte));
        }

        set_state!(self, State::ResponseVersionMinor, response_version_minor);
        change_state_fast!(self, context);
    }

    #[inline]
    pub fn response_version_minor(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit: u16 = (get_length!(self) & MINOR_MASK) as u16;

        collect_digits!(self, context, digit, 999, ParserError::Version, ERR_VERSION, {
            inc_length!(self, digit as u64);

            exit_eof!(self, context);
        });

        set_state!(self, State::StripResponseStatusCode, strip_response_status_code);

        if context.handler.on_version((get_length!(self) >> MAJOR_SHIFT) as u16, digit) {
            change_state_fast!(self, context);
        } else {
            exit_callback!(self, context);
        }
    }

    #[inline]
    pub fn strip_response_status_code(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_space_tab!(self, context);

        if !is_digit!(context.byte) {
            exit_error!(self, context, ParserError::StatusCode(ERR_STATUS_CODE, context.byte));
        }

        replay!(context);

        set_length!(self, 0);

        set_state!(self, State::ResponseStatusCode, response_status_code);
        change_state_fast!(self, context);
    }

    #[inline]
    pub fn response_status_code(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = get_length!(self) as u16;

        collect_digits!(self, context, digit, 999, ParserError::StatusCode, ERR_STATUS_CODE, {
            set_length!(self, digit as u64);
            exit_eof!(self, context);
        });

        replay!(context);
        set_state!(self, State::StripResponseStatus, strip_response_status);

        if context.handler.on_status_code(digit as u16) {
            change_state_fast!(self, context);
        } else {
            exit_callback!(self, context);
        }
    }

    #[inline]
    pub fn strip_response_status(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_space_tab!(self, context);
        replay!(context);
        set_state!(self, State::ResponseStatus, response_status);
        change_state_fast!(self, context);
    }

    #[inline]
    pub fn response_status(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        loop {
            if is_eof!(context) {
                callback_or_eof!(self, context, on_status);
            }

            next!(context);

            if context.byte == b'\r' {
                break;
            } else if context.byte != b' ' && context.byte != b'\t' && !is_token(context.byte) {
                exit_error!(self, context, ParserError::Status(ERR_STATUS, context.byte));
            }
        }

        set_state!(self, State::PreHeaders1, pre_headers1);
        callback_ignore!(self, context, on_status, {
            change_state_fast!(self, context);
        });
    }

    // ---------------------------------------------------------------------------------------------
    // BODY STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    pub fn body(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        if context.handler.get_transfer_encoding() == TransferEncoding::Chunked {
            set_flag!(self, F_CHUNKED);
            set_state!(self, State::ChunkSize1, chunk_size1);
            change_state!(self, context);
        }

        exit_eof!(self, context);
    }

    #[inline]
    pub fn content(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn chunk_size1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        match hex_to_byte(&[context.byte]) {
            Some(byte) => {
                set_length!(self, byte as u64);

                set_state!(self, State::ChunkSize2, chunk_size2);
                change_state_fast!(self, context);
            },
            None => {
                exit_error!(self, context, ParserError::ChunkSize(ERR_CHUNK_SIZE, context.byte));
            }
        }
    }

    #[inline]
    pub fn chunk_size2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        chunk_size!(self, context, State::ChunkSize3, chunk_size3);
    }

    #[inline]
    pub fn chunk_size3(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        chunk_size!(self, context, State::ChunkSize4, chunk_size4);
    }

    #[inline]
    pub fn chunk_size4(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        chunk_size!(self, context, State::ChunkSize5, chunk_size5);
    }

    #[inline]
    pub fn chunk_size5(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        chunk_size!(self, context, State::ChunkSize6, chunk_size6);
    }

    #[inline]
    pub fn chunk_size6(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        chunk_size!(self, context, State::ChunkSize7, chunk_size7);
    }

    #[inline]
    pub fn chunk_size7(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        chunk_size!(self, context, State::ChunkSize8, chunk_size8);
    }

    #[inline]
    pub fn chunk_size8(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        chunk_size!(self, context, State::ChunkSize9, chunk_size9);
    }

    #[inline]
    pub fn chunk_size9(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        chunk_size!(self, context, State::ChunkSize10, chunk_size10);
    }

    #[inline]
    pub fn chunk_size10(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        // this must go back to itself in order to consume the
        // remaining carriage return and finalize the chunk size
        chunk_size!(self, context, State::ChunkSize10, chunk_size10);
    }

    #[inline]
    pub fn chunk_extension(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn chunk_size_newline(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'\n' {
            set_state!(self, State::ChunkData, chunk_data);
            change_state!(self, context);
        }

        exit_error!(self, context, ParserError::CrlfSequence(ERR_CRLF_SEQUENCE, context.byte));
    }

    #[inline]
    pub fn chunk_data(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn chunk_data_newline1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn chunk_data_newline2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn multipart_hyphen1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn multipart_hyphen2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn multipart_boundary(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn multipart_newline1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn multipart_newline2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn multipart_data(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn urlencoded_field(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn urlencoded_field_ampersand(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn urlencoded_field_hex(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn urlencoded_field_plus(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn urlencoded_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn urlencoded_value_hex(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn urlencoded_value_plus(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn urlencoded_newline1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn urlencoded_newline2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }
}
