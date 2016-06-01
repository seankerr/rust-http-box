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

#![allow(dead_code)]

use byte::hex_to_byte;
use byte::is_token;

use std::{ fmt,
           str };

// State flag mask.
const FLAG_MASK: u64 = 0x7F;

// State flag shift.
const FLAG_SHIFT: u8 = 1;

// Lower 8 bits mask.
const LOWER8_MASK: u64 = 0xFF;

// Lower 8 bits shift.
const LOWER8_SHIFT: u8 = 8;

// Lower 16 bits mask.
const LOWER16_MASK: u64 = 0xFFFF;

// Lower 16 bits shift.
const LOWER16_SHIFT: u8 = 8;

// Mid 8 bits mask.
const MID8_MASK: u64 = 0xFF;

// Mid 8 bits shift.
const MID8_SHIFT: u8 = 16;

// Upper 40 bits mask.
const UPPER40_MASK: u64 = 0xFFFFFFFFFF;

// Upper 40 bits shift.
const UPPER40_SHIFT: u8 = 24;

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
// BIT DATA MACROS
// -------------------------------------------------------------------------------------------------

// Retrieve the lower 8 bits.
macro_rules! get_lower8 {
    ($parser:expr) => ({
        (($parser.bit_data >> LOWER8_SHIFT) & LOWER8_MASK) as u8
    });
}

// Retrieve the lower 16 bits.
macro_rules! get_lower16 {
    ($parser:expr) => ({
        (($parser.bit_data >> LOWER16_SHIFT) & LOWER16_MASK) as u16
    });
}

// Retrieve the mid 8 bits.
macro_rules! get_mid8 {
    ($parser:expr) => ({
        (($parser.bit_data >> MID8_SHIFT) & MID8_MASK) as u8
    });
}

// Retrieve the upper 40 bits.
macro_rules! get_upper40 {
    ($parser:expr) => ({
        ($parser.bit_data >> UPPER40_SHIFT) & UPPER40_MASK
    });
}

// Indicates that a state flag is set.
macro_rules! has_flag {
    ($parser:expr, $flag:expr) => ({
        (($parser.bit_data >> FLAG_SHIFT) & FLAG_MASK) & $flag.bits == $flag.bits
    });
}

// Set a state flag.
macro_rules! set_flag {
    ($parser:expr, $flag:expr) => ({
        $parser.bit_data |= ($flag.bits & FLAG_MASK) << FLAG_SHIFT;
    });
}

// Set the lower 8 bits.
macro_rules! set_lower8 {
    ($parser:expr, $bits:expr) => ({
        let bits = $bits as u64;

        $parser.bit_data &= !(LOWER8_MASK << LOWER8_SHIFT);
        $parser.bit_data |= bits << LOWER8_SHIFT;
    });
}

// Set the mid 8 bits.
macro_rules! set_mid8 {
    ($parser:expr, $bits:expr) => ({
        let bits = $bits as u64;

        $parser.bit_data &= !(MID8_MASK << MID8_SHIFT);
        $parser.bit_data |= bits << MID8_SHIFT;
    });
}

// Set the lower 16 bits.
macro_rules! set_lower16 {
    ($parser:expr, $bits:expr) => ({
        let bits = $bits as u64;

        $parser.bit_data &= !(LOWER16_MASK << LOWER16_SHIFT);
        $parser.bit_data |= bits << LOWER16_SHIFT;
    });
}

// Set the upper 40 bits.
macro_rules! set_upper40 {
    ($parser:expr, $bits:expr) => ({
        let bits = $bits as u64;

        $parser.bit_data &= !(UPPER40_MASK << UPPER40_SHIFT);
        $parser.bit_data |= bits << UPPER40_SHIFT;
    });
}

// Unset a state flag.
macro_rules! unset_flag {
    ($parser:expr, $flag:expr) => ({
        $parser.bit_data &= !(($flag.bits & FLAG_MASK) << FLAG_SHIFT);
    });
}

// -------------------------------------------------------------------------------------------------
// STREAM MACROS
// -------------------------------------------------------------------------------------------------

// Execute callback `$function`. If it returns `true`, execute `$exec`. Otherwise exit with
// `Success::Callback`.
macro_rules! callback {
    ($parser:expr, $context:expr, $function:ident, $data:expr, $exec:expr) => ({
        if $context.handler.$function($data) {
            $exec
        } else {
            exit_callback!($parser, $context);
        }
    });

    ($parser:expr, $context:expr, $function:ident, $exec:expr) => ({
        let slice = stream_collected_bytes!($context);

        if slice.len() > 0 {
            if $context.handler.$function(slice) {
                $exec
            } else {
                exit_callback!($parser, $context);
            }
        } else {
            $exec
        }
    });
}

// Reusable callback EOS expression that executes `$function`.
macro_rules! callback_eos_expr {
    ($parser:expr, $context:expr, $function:ident) => ({
        callback!($parser, $context, $function, {
            exit_eos!($parser, $context);
        });
    });
}

// Execute callback `$function` ignoring the last collected byte. If it returns `true`, transition
// to `$state`. Otherwise exit with `Success::Callback`.
macro_rules! callback_ignore_transition {
    ($parser:expr, $context:expr, $function:ident, $state:expr, $state_function:ident) => ({
        let slice = &$context.stream[$context.mark_index..$context.stream_index - 1];

        set_state!($parser, $state, $state_function);

        if slice.len() > 0 {
            if $context.handler.$function(slice) {
                transition!($parser, $context);
            } else {
                exit_callback!($parser, $context);
            }
        } else {
            transition!($parser, $context);
        }
    });
}

// Execute callback `$function` ignoring the last collected byte. If it returns `true`, transition
// to the next `$state` quickly by directly calling `$state_function`. Otherwise exit with
// `Success::Callback`.
macro_rules! callback_ignore_transition_fast {
    ($parser:expr, $context:expr, $function:ident, $state:expr, $state_function:ident) => ({
        let slice = &$context.stream[$context.mark_index..$context.stream_index - 1];

        set_state!($parser, $state, $state_function);

        if slice.len() > 0 {
            if $context.handler.$function(slice) {
                transition_fast!($parser, $context);
            } else {
                exit_callback!($parser, $context);
            }
        } else {
            transition_fast!($parser, $context);
        }
    });
}

// Execute callback `$function`. If it returns `true`, transition to the `$state`. Otherwise exit
// with `Success::Callback`.
//
// This macro exists to enforce the design decision that after each callback, state must either
// change, or the parser must exit with `Success::Callback`.
macro_rules! callback_transition {
    ($parser:expr, $context:expr, $function:ident, $data:expr, $state:expr,
     $state_function:ident) => ({
        set_state!($parser, $state, $state_function);
        callback!($parser, $context, $function, $data, {
            transition!($parser, $context);
        });
    });

    ($parser:expr, $context:expr, $function:ident, $state:expr, $state_function:ident) => ({
        set_state!($parser, $state, $state_function);
        callback!($parser, $context, $function, {
            transition!($parser, $context);
        });
    });
}

// Execute callback `$function`. If it returns `true`, transition to the `$state` quickly by
// directly calling `$state_function`. Otherwise exit with `Success::Callback`.
//
// This macro exists to enforce the design decision that after each callback, state must either
// change, or the parser must exit with `Success::Callback`.
macro_rules! callback_transition_fast {
    ($parser:expr, $context:expr, $function:ident, $data:expr, $state:expr,
     $state_function:ident) => ({
        set_state!($parser, $state, $state_function);
        callback!($parser, $context, $function, $data, {
            transition_fast!($parser, $context);
        });
    });

    ($parser:expr, $context:expr, $function:ident, $state:expr, $state_function:ident) => ({
        set_state!($parser, $state, $state_function);
        callback!($parser, $context, $function, {
            transition_fast!($parser, $context);
        });
    });
}

// Collect remaining bytes until content length is zero.
//
// Content length is stored in the upper 40 bits.
macro_rules! collect_content_length {
    ($parser:expr, $context:expr) => ({
        exit_if_eos!($parser, $context);

        if stream_has_bytes!($context, get_upper40!($parser) as usize) {
            $context.stream_index += get_upper40!($parser) as usize;

            set_upper40!($parser, 0);

            true
        } else {
            $context.stream_index += $context.stream.len();

            set_upper40!($parser, get_upper40!($parser) as usize - $context.stream.len());

            false
        }
    });
}

// Collect header value.
macro_rules! collect_header_value {
    ($context:expr, $error:expr, $eos:expr) => ({
        stream_collect!($context, $eos, {
            if $context.byte == b'\r' {
                break;
            } else if $context.byte > 0x1F && $context.byte < 0x7F {
                // space + visible
                continue;
            }

            return Err($error($context.byte));
        });
    });
}

// Collect all bytes that are allowed within a quoted value.
macro_rules! collect_quoted_value {
    ($parser:expr, $context:expr, $error:expr, $function:ident) => ({
        stream_collect!($context,
            callback_eos_expr!($parser, $context, $function),
            if b'"' == $context.byte || b'\\' == $context.byte {
                break;
            } else if is_non_visible!($context.byte) && $context.byte != b' ' {
                return Err($error($context.byte));
            }
        );
    });
}

// Consume all linear white space until a non-linear white space byte is found.
macro_rules! consume_linear_space {
    ($parser:expr, $context:expr) => ({
        loop {
            if stream_is_eos!($context) {
                exit_eos!($parser, $context);
            }

            stream_next!($context);

            if $context.byte == b' ' || $context.byte == b'\t' {
                continue;
            } else {
                break;
            }
        }
    });
}

// Exit parser with `Success::Callback`.
macro_rules! exit_callback {
    ($parser:expr, $context:expr) => ({
        return Ok(ParserValue::Exit(Success::Callback($context.stream_index)));
    });
}

// Exit parser with `Success::Eos`.
macro_rules! exit_eos {
    ($parser:expr, $context:expr) => ({
        return Ok(ParserValue::Exit(Success::Eos($context.stream_index)));
    });
}

// Exit parser with `Success::Finished`.
macro_rules! exit_finished {
    ($parser:expr, $context:expr) => ({
        return Ok(ParserValue::Exit(Success::Finished($context.stream_index)));
    });
}

// If the stream is EOS, exit with `Success::Eos`. Otherwise do nothing.
macro_rules! exit_if_eos {
    ($parser:expr, $context:expr) => ({
        if stream_is_eos!($context) {
            exit_eos!($parser, $context);
        }
    });
}

// Set state and state function.
macro_rules! set_state {
    ($parser:expr, $state:expr, $state_function:ident) => ({
        $parser.state          = $state;
        $parser.state_function = Parser::$state_function;
    });
}

// Transition to `$state`.
macro_rules! transition {
    ($parser:expr, $context:expr, $state:expr, $state_function:ident) => ({
        set_state!($parser, $state, $state_function);

        $context.mark_index = $context.stream_index;

        return Ok(ParserValue::Continue);
    });

    ($parser:expr, $context:expr) => ({
        $context.mark_index = $context.stream_index;

        return Ok(ParserValue::Continue);
    });
}

// Transition to `$state` quickly by directly calling `$state_function`.
macro_rules! transition_fast {
    ($parser:expr, $context:expr, $state:expr, $state_function:ident) => ({
        set_state!($parser, $state, $state_function);

        $context.mark_index = $context.stream_index;

        return ($parser.state_function)($parser, $context);
    });

    ($parser:expr, $context:expr) => ({
        $context.mark_index = $context.stream_index;

        return ($parser.state_function)($parser, $context);
    });
}

// -------------------------------------------------------------------------------------------------

/// Parser state function type.
pub type StateFunction<T> = fn(&mut Parser<T>, &mut ParserContext<T>)
    -> Result<ParserValue, ParserError>;

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

impl fmt::Debug for Connection {
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

impl fmt::Display for Connection {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Connection::None => {
                write!(formatter, "None")
            },
            Connection::Close => {
                write!(formatter, "Close")
            },
            Connection::KeepAlive => {
                write!(formatter, "KeepAlive")
            },
            Connection::Upgrade => {
                write!(formatter, "Upgrade")
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

impl fmt::Debug for ContentLength {
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

impl fmt::Display for ContentLength {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ContentLength::None => {
                write!(formatter, "None")
            },
            ContentLength::Specified(x) => {
                write!(formatter, "{}", x)
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

impl fmt::Debug for ContentType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ContentType::None => {
                write!(formatter, "None")
            },
            ContentType::Multipart(ref x) => {
                write!(formatter, "ContentType::Multipart({})",
                       str::from_utf8((*x).as_slice()).unwrap())
            },
            ContentType::UrlEncoded => {
                write!(formatter, "ContentType::UrlEncoded")
            },
            ContentType::Other(ref x) => {
                write!(formatter, "ContentType::Other({})",
                       str::from_utf8((*x).as_slice()).unwrap())
            }
        }
    }
}

impl fmt::Display for ContentType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ContentType::None => {
                write!(formatter, "None")
            },
            ContentType::Multipart(ref x) => {
                write!(formatter, "{}", str::from_utf8((*x).as_slice()).unwrap())
            },
            ContentType::UrlEncoded => {
                write!(formatter, "UrlEncoded")
            },
            ContentType::Other(ref x) => {
                write!(formatter, "{}", str::from_utf8((*x).as_slice()).unwrap())
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Parser error messages.
#[derive(Clone,Copy,PartialEq)]
pub enum ParserError {
    /// Invalid chunk extension name.
    ChunkExtensionName(u8),

    /// Invalid chunk extension value.
    ChunkExtensionValue(u8),

    /// Invalid chunk size.
    ChunkSize(u8),

    /// Invalid CRLF sequence.
    CrlfSequence(u8),

    /// Parsing has failed, but `Parser::parse()` is executed again.
    Dead,

    /// Invalid header field.
    HeaderField(u8),

    /// Invalid header value.
    HeaderValue(u8),

    /// Maximum content length has been met.
    MaxContentLength,

    /// Maximum multipart boundary length has been met.
    MaxMultipartBoundaryLength,

    /// Missing an expected Content-Length header.
    MissingContentLength,

    /// Invalid request method.
    Method(u8),

    /// Invalid multipart boundary.
    MultipartBoundary(u8),

    /// Invalid status.
    Status(u8),

    /// Invalid status code.
    StatusCode(u8),

    /// Invalid URL character.
    Url(u8),

    /// Invalid URL encoded field.
    UrlEncodedField(u8),

    /// Invalid URL encoded value.
    UrlEncodedValue(u8),

    /// Invalid HTTP version.
    Version(u8),
}

impl fmt::Display for ParserError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserError::ChunkExtensionName(ref byte) => {
                write!(formatter, "Invalid chunk extension name at byte {}", byte)
            },
            ParserError::ChunkExtensionValue(ref byte) => {
                write!(formatter, "Invalid chunk extension value at byte {}", byte)
            },
            ParserError::ChunkSize(ref byte) => {
                write!(formatter, "Invalid chunk size at byte {}", byte)
            },
            ParserError::CrlfSequence(ref byte) => {
                write!(formatter, "Invalid CRLF sequence at byte {}", byte)
            },
            ParserError::Dead => {
                write!(formatter, "Parser is dead")
            },
            ParserError::HeaderField(ref byte) => {
                write!(formatter, "Invalid header field at byte {}", byte)
            },
            ParserError::HeaderValue(ref byte) => {
                write!(formatter, "Invalid header value at byte {}", byte)
            },
            ParserError::MaxContentLength => {
                write!(formatter, "Maximum content length exceeded")
            },
            ParserError::MaxMultipartBoundaryLength => {
                write!(formatter, "Maximum multipart boundary size exceeded")
            },
            ParserError::MissingContentLength => {
                write!(formatter, "Missing content length")
            },
            ParserError::Method(ref byte) => {
                write!(formatter, "Invalid method at byte {}", byte)
            },
            ParserError::MultipartBoundary(ref byte) => {
                write!(formatter, "Invalid multipart boundary at byte {}", byte)
            },
            ParserError::Status(ref byte) => {
                write!(formatter, "Invalid status at byte {}", byte)
            },
            ParserError::StatusCode(ref byte) => {
                write!(formatter, "Invalid status code at byte {}", byte)
            },
            ParserError::Url(ref byte) => {
                write!(formatter, "Invalid URL at byte {}", byte)
            },
            ParserError::UrlEncodedField(ref byte) => {
                write!(formatter, "Invalid URL encoded field at byte {}", byte)
            },
            ParserError::UrlEncodedValue(ref byte) => {
                write!(formatter, "Invalid URL encoded value at byte {}", byte)
            },
            ParserError::Version(ref byte) => {
                write!(formatter, "Invalid HTTP version at byte {}", byte)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Parser return values.
pub enum ParserValue {
    /// Continue the parser loop.
    Continue,

    /// Exit the parser loop.
    Exit(Success)
}

/// Parser states.
///
/// These states are in the order that they are processed.
#[derive(Clone,Copy,Debug,PartialEq)]
#[repr(u8)]
pub enum State {
    /// An error was returned from a call to `Parser::parse()`.
    Dead = 1,

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

    /// Parsing body.
    Body,

    /// Unparsable content.
    Content,

    /// Parsing chunk size.
    ChunkSize,

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

impl fmt::Debug for TransferEncoding {
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
                write!(formatter, "TransferEncoding::Other({})",
                       str::from_utf8((*x).as_slice()).unwrap())
            }
        }
    }
}

impl fmt::Display for TransferEncoding {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TransferEncoding::None => {
                write!(formatter, "None")
            },
            TransferEncoding::Chunked => {
                write!(formatter, "Chunked")
            },
            TransferEncoding::Compress => {
                write!(formatter, "Compress")
            },
            TransferEncoding::Deflate => {
                write!(formatter, "Deflate")
            },
            TransferEncoding::Gzip => {
                write!(formatter, "Gzip")
            },
            TransferEncoding::Other(ref x) => {
                write!(formatter, "{}", str::from_utf8((*x).as_slice()).unwrap())
            }
        }
    }
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

    /// Callback that is executed when parsing a URL fragment has completed.
    ///
    /// This may be executed multiple times in order to supply the entire fragment.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_fragment(&mut self, fragment: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when parsing a URL host has completed.
    ///
    /// This may be executed multiple times in order to supply the entire fragment.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_host(&mut self, host: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when parsing a URL path has completed.
    ///
    /// This may be executed multiple times in order to supply the entire path.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_path(&mut self, path: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when parsing a URL port has completed.
    ///
    /// This may be executed multiple times in order to supply the entire port.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_port(&mut self, port: u16) -> bool {
        true
    }

    /// Callback that is executed when parsing a URL query string has completed.
    ///
    /// This may be executed multiple times in order to supply the entire query string.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_query_string(&mut self, query_string: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when parsing a URL scheme has completed.
    ///
    /// This may be executed multiple times in order to supply the entire scheme.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_scheme(&mut self, scheme: &[u8]) -> bool {
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
pub struct ParserContext<'a, T: HttpHandler + 'a> {
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

/// Parser data.
pub struct Parser<T: HttpHandler> {
    // Bit data that stores parser bit details.
    //
    // Bits 1-8: State flags that are checked when states have a dual purpose, such as when header
    //           parsing states also parse chunk encoding trailers.
    // Macros:   has_flag!(), set_flag!(), unset_flag!()
    //
    // Bits 5-64: Used to store various numbers depending on state. Content length, chunk size,
    //            HTTP major/minor versions are all stored in here. Depending on macro used, more
    //            bits are accessible.
    // Macros:    get_lower8!(), set_lower8!()   -- lower 8 bits
    //            get_mid8!(), set_mid8!()       -- mid 8 bits
    //            get_lower16!(), set_lower16!() -- lower 16 bits
    //                                              (when not using the lower8/mid8 macros)
    //            get_upper40!(), set_upper40!() -- upper 40 bits
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

impl<T: HttpHandler> Parser<T> {
    /// Create a new `Parser`.
    fn new(state: State, state_function: StateFunction<T>) -> Parser<T> {
        Parser{ bit_data:       if state == State::StripRequestMethod {
                                    1
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
    pub fn parse(&mut self, handler: &mut T, stream: &[u8]) -> Result<Success, ParserError> {
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
                    self.byte_count += context.stream_index;
                    self.state       = State::Dead;

                    return Err(error);
                }
            }
        }
    }

    /// Reset the `Parser` back to its original state.
    pub fn reset(&mut self) {
        self.byte_count   = 0;
        self.content_type = ContentType::None;

        if self.bit_data & 1 == 1 {
            self.bit_data       = 1;
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
    fn pre_headers1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        stream_next!(context);

        if context.byte == b'\n' {
            transition_fast!(self, context, State::PreHeaders2, pre_headers2);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn pre_headers2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        stream_next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, State::Newline4, newline4);
        } else {
            stream_replay!(context);

            transition_fast!(self, context, State::StripHeaderField, strip_header_field);
        }
    }

    #[inline]
    fn strip_header_field(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);
        stream_replay!(context);

        transition_fast!(self, context, State::FirstHeaderField, first_header_field);
    }

    #[inline]
    #[cfg_attr(test, allow(cyclomatic_complexity))]
    fn first_header_field(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        macro_rules! field {
            ($header:expr, $length:expr) => ({
                stream_jump!(context, $length);

                callback_transition_fast!(self, context,
                                          on_header_field, $header,
                                          State::StripHeaderValue, strip_header_value);
            });
        }

        if stream_has_bytes!(context, 26) {
            // have enough bytes to compare common header fields immediately, without collecting
            // individual tokens
            if context.byte == b'C' {
                if b"Connection:" == stream_peek!(context, 11) {
                    field!(b"Connection", 11);
                } else if b"Content-Type:" == stream_peek!(context, 13) {
                    field!(b"Content-Type", 13);
                } else if b"Content-Length:" == stream_peek!(context, 15) {
                    field!(b"Content-Length", 15);
                } else if b"Cookie:" == stream_peek!(context, 7) {
                    field!(b"Cookie", 7);
                } else if b"Cache-Control:" == stream_peek!(context, 14) {
                    field!(b"Cache-Control", 14);
                } else if b"Content-Security-Policy:" == stream_peek!(context, 24) {
                    field!(b"Content-Security-Policy", 24);
                }
            } else if context.byte == b'A' {
                if b"Accept:" == stream_peek!(context, 7) {
                    field!(b"Accept", 7);
                } else if b"Accept-Charset:" == stream_peek!(context, 15) {
                    field!(b"Accept-Charset", 15);
                } else if b"Accept-Encoding:" == stream_peek!(context, 16) {
                    field!(b"Accept-Encoding", 16);
                } else if b"Accept-Language:" == stream_peek!(context, 16) {
                    field!(b"Accept-Language", 16);
                } else if b"Authorization:" == stream_peek!(context, 14) {
                    field!(b"Authorization", 14);
                }
            } else if context.byte == b'L' {
                if b"Location:" == stream_peek!(context, 9) {
                    field!(b"Location", 9);
                } else if b"Last-Modified:" == stream_peek!(context, 14) {
                    field!(b"Last-Modified", 14);
                }
            } else if context.byte == b'P' && b"Pragma:" == stream_peek!(context, 7) {
                field!(b"Pragma", 7);
            } else if context.byte == b'S' && b"Set-Cookie:" == stream_peek!(context, 11) {
                field!(b"Set-Cookie", 11);
            } else if context.byte == b'T' && b"Transfer-Encoding:" == stream_peek!(context, 18) {
                field!(b"Transfer-Encoding", 18);
            } else if context.byte == b'U' {
                if b"User-Agent:" == stream_peek!(context, 11) {
                    field!(b"User-Agent", 11);
                } else if b"Upgrade:" == stream_peek!(context, 8) {
                    field!(b"Upgrade", 8);
                }
            } else if context.byte == b'X' {
                if b"X-Powered-By:" == stream_peek!(context, 13) {
                    field!(b"X-Powered-By", 13);
                } else if b"X-Forwarded-For:" == stream_peek!(context, 16) {
                    field!(b"X-Forwarded-For", 16);
                } else if b"X-Forwarded-Host:" == stream_peek!(context, 17) {
                    field!(b"X-Forwarded-Host", 17);
                } else if b"X-XSS-Protection:" == stream_peek!(context, 17) {
                    field!(b"X-XSS-Protection", 17);
                } else if b"X-WebKit-CSP:" == stream_peek!(context, 13) {
                    field!(b"X-WebKit-CSP", 13);
                } else if b"X-Content-Security-Policy:" == stream_peek!(context, 26) {
                    field!(b"X-Content-Security-Policy", 26);
                }
            } else if context.byte == b'W' && b"WWW-Authenticate:" == stream_peek!(context, 17) {
                field!(b"WWW-Authenticate", 17);
            }
        }

        transition_fast!(self, context, State::HeaderField, header_field);
    }

    #[inline]
    fn header_field(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        stream_collect_tokens!(context, ParserError::HeaderField,
            callback_eos_expr!(self, context, on_header_field),

            // stop on these bytes
            context.byte == b':'
        );

        callback_ignore_transition_fast!(self, context,
                                         on_header_field,
                                         State::StripHeaderValue, strip_header_value);
    }

    #[inline]
    fn strip_header_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);

        if context.byte == b'"' {
            transition_fast!(self, context, State::HeaderQuotedValue, header_quoted_value);
        }

        stream_replay!(context);

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
        stream_next!(context);

        callback_transition!(self, context,
                             on_header_value, &[context.byte],
                             State::HeaderQuotedValue, header_quoted_value);
    }

    #[inline]
    fn newline1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        if stream_has_bytes!(context, 2) && b"\r\n" == stream_peek!(context, 2) {
            transition_fast!(self, context, State::Newline3, newline3);
        }

        exit_if_eos!(self, context);
        stream_next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, State::Newline2, newline2);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn newline2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        stream_next!(context);

        if context.byte == b'\n' {
            transition_fast!(self, context, State::Newline3, newline3);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn newline3(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        stream_next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, State::Newline4, newline4);
        } else if context.byte == b' ' || context.byte == b'\t' {
            // multiline header value
            callback_transition!(self, context,
                                 on_header_value, b" ",
                                 State::StripHeaderValue, strip_header_value);
        } else {
            stream_replay!(context);
            transition!(self, context, State::StripHeaderField, strip_header_field);
        }
    }

    #[inline]
    fn newline4(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        stream_next!(context);

        if context.byte == b'\n' {
            if has_flag!(self, F_CHUNKED) {
                context.handler.on_headers_finished();

                transition_fast!(self, context, State::Finished, finished);
            }

            set_state!(self, State::Body, body);

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
    fn strip_request_method(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);
        stream_replay!(context);

        transition_fast!(self, context, State::RequestMethod, request_method);
    }

    #[inline]
    fn request_method(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        macro_rules! method {
            ($method:expr, $length:expr) => (
                stream_jump!(context, $length);

                callback_transition_fast!(self, context,
                                          on_method, $method,
                                          State::StripRequestUrl, strip_request_url);
            );
        }

        if stream_has_bytes!(context, 8) {
            // have enough bytes to compare all known methods immediately, without collecting
            // individual tokens

            // get the first byte, then replay it (for use with stream_peek!())
            stream_next!(context);
            stream_replay!(context);

            if context.byte == b'G' && b"GET " == stream_peek!(context, 4) {
                method!(b"GET", 4);
            } else if context.byte == b'P' {
                if b"POST " == stream_peek!(context, 5) {
                    method!(b"POST", 5);
                } else if b"PUT " == stream_peek!(context, 4) {
                    method!(b"PUT", 4);
                }
            } else if context.byte == b'D' && b"DELETE " == stream_peek!(context, 7) {
                method!(b"DELETE", 7);
            } else if context.byte == b'C' && b"CONNECT " == stream_peek!(context, 8) {
                method!(b"CONNECT", 8);
            } else if context.byte == b'O' && b"OPTIONS " == stream_peek!(context, 8) {
                method!(b"OPTIONS", 8);
            } else if context.byte == b'H' && b"HEAD " == stream_peek!(context, 5) {
                method!(b"HEAD", 5);
            } else if context.byte == b'T' && b"TRACE " == stream_peek!(context, 6) {
                method!(b"TRACE", 6);
            }
        }

        stream_collect_tokens!(context, ParserError::Method,
            callback_eos_expr!(self, context, on_method),

            // stop on these bytes
            context.byte == b' '
        );

        stream_replay!(context);

        callback_transition_fast!(self, context,
                                  on_method,
                                  State::StripRequestUrl, strip_request_url);
    }

    #[inline]
    fn strip_request_url(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);
        stream_replay!(context);

        transition_fast!(self, context, State::RequestUrl, request_url);
    }

    #[inline]
    fn request_url(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        stream_collect_visible!(context, ParserError::Url,
            callback_eos_expr!(self, context, on_url),

            // stop on these bytes
            context.byte == b' '
        );

        stream_replay!(context);

        callback_transition_fast!(self, context,
                                  on_url,
                                  State::StripRequestHttp, strip_request_http);
    }

    #[inline]
    fn strip_request_http(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);
        stream_replay!(context);

        transition_fast!(self, context, State::RequestHttp1, request_http1);
    }

    #[inline]
    fn request_http1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        macro_rules! version {
            ($major:expr, $minor:expr, $length:expr) => (
                stream_jump!(context, $length);
                set_state!(self, State::PreHeaders1, pre_headers1);

                if context.handler.on_version($major, $minor) {
                    transition_fast!(self, context);
                } else {
                    exit_callback!(self, context);
                }
            );
        }

        if stream_has_bytes!(context, 9) {
            // have enough bytes to compare all known versions immediately, without collecting
            // individual tokens
            if b"HTTP/1.1\r" == stream_peek!(context, 9) {
                version!(1, 1, 9);
            } else if b"HTTP/2.0\r" == stream_peek!(context, 9) {
                version!(2, 0, 9);
            } else if b"HTTP/1.0\r" == stream_peek!(context, 9) {
                version!(1, 0, 9);
            }
        }

        exit_if_eos!(self, context);
        stream_next!(context);

        if context.byte == b'H' || context.byte == b'h' {
            transition_fast!(self, context, State::RequestHttp2, request_http2);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_http2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        stream_next!(context);

        if context.byte == b'T' || context.byte == b't' {
            transition_fast!(self, context, State::RequestHttp3, request_http3);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_http3(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        stream_next!(context);

        if context.byte == b'T' || context.byte == b't' {
            transition_fast!(self, context, State::RequestHttp4, request_http4);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_http4(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        stream_next!(context);

        if context.byte == b'P' || context.byte == b'p' {
            transition_fast!(self, context, State::RequestHttp5, request_http5);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_http5(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        stream_next!(context);

        if context.byte == b'/' {
            set_upper40!(self, 0);

            transition_fast!(self, context, State::RequestVersionMajor, request_version_major);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_version_major(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = get_lower16!(self) as u64;

        stream_collect_digits!(context, ParserError::Version, digit, 999, {
            set_lower16!(self, digit as u16);

            exit_eos!(self, context);
        });

        set_lower16!(self, digit as u16);

        if context.byte == b'.' {
            transition_fast!(self, context, State::RequestVersionMinor, request_version_minor);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_version_minor(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = get_upper40!(self);

        stream_collect_digits!(context, ParserError::Version, digit, 999, {
            set_upper40!(self, digit);

            exit_eos!(self, context);
        });

        set_state!(self, State::PreHeaders1, pre_headers1);

        if context.handler.on_version(get_lower16!(self), digit as u16) {
            transition_fast!(self, context);
        } else {
            exit_callback!(self, context);
        }
    }

    // ---------------------------------------------------------------------------------------------
    // RESPONSE STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn strip_response_http(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);
        stream_replay!(context);

        transition_fast!(self, context, State::ResponseHttp1, response_http1);
    }

    #[inline]
    fn response_http1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        macro_rules! version {
            ($major:expr, $minor:expr, $length:expr) => (
                stream_jump!(context, $length);
                set_state!(self, State::StripResponseStatusCode, strip_response_status_code);

                if context.handler.on_version($major, $minor) {
                    transition_fast!(self, context);
                } else {
                    exit_callback!(self, context);
                }
            );
        }

        if stream_has_bytes!(context, 9) {
            // have enough bytes to compare all known versions immediately, without collecting
            // individual tokens
            if b"HTTP/1.1 " == stream_peek!(context, 9) {
                version!(1, 1, 9);
            } else if b"HTTP/2.0 " == stream_peek!(context, 9) {
                version!(2, 0, 9);
            } else if b"HTTP/1.0 " == stream_peek!(context, 9) {
                version!(1, 0, 9);
            }
        }

        exit_if_eos!(self, context);
        stream_next!(context);

        if context.byte == b'H' || context.byte == b'h' {
            transition_fast!(self, context, State::ResponseHttp2, response_http2);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn response_http2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        stream_next!(context);

        if context.byte == b'T' || context.byte == b't' {
            transition_fast!(self, context, State::ResponseHttp3, response_http3);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn response_http3(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        stream_next!(context);

        if context.byte == b'T' || context.byte == b't' {
            transition_fast!(self, context, State::ResponseHttp4, response_http4);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn response_http4(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        stream_next!(context);

        if context.byte == b'P' || context.byte == b'p' {
            transition_fast!(self, context, State::ResponseHttp5, response_http5);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn response_http5(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        stream_next!(context);

        if context.byte == b'/' {
            set_upper40!(self, 0);

            transition_fast!(self, context, State::ResponseVersionMajor, response_version_major);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn response_version_major(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = get_lower16!(self) as u64;

        stream_collect_digits!(context, ParserError::Version, digit, 999, {
            set_lower16!(self, digit as u16);

            exit_eos!(self, context);
        });

        set_lower16!(self, digit as u16);

        if context.byte == b'.' {
            transition_fast!(self, context, State::ResponseVersionMinor, response_version_minor);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn response_version_minor(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = get_upper40!(self);

        stream_collect_digits!(context, ParserError::Version, digit, 999, {
            set_upper40!(self, digit);

            exit_eos!(self, context);
        });

        set_state!(self, State::StripResponseStatusCode, strip_response_status_code);

        if context.handler.on_version(get_lower16!(self), digit as u16) {
            transition_fast!(self, context);
        } else {
            exit_callback!(self, context);
        }
    }

    #[inline]
    fn strip_response_status_code(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);

        if !is_digit!(context.byte) {
            return Err(ParserError::StatusCode(context.byte));
        }

        stream_replay!(context);

        set_upper40!(self, 0);

        transition_fast!(self, context, State::ResponseStatusCode, response_status_code);
    }

    #[inline]
    fn response_status_code(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = get_upper40!(self);

        stream_collect_digits!(context, ParserError::StatusCode, digit, 999, {
            set_upper40!(self, digit);
            exit_eos!(self, context);
        });

        stream_replay!(context);
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
        stream_replay!(context);

        transition_fast!(self, context, State::ResponseStatus, response_status);
    }

    #[inline]
    fn response_status(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        stream_collect!(context, {
            callback!(self, context, on_status, {
                exit_eos!(self, context);
            });
        }, {
            if context.byte == b'\r' {
                break;
            } else if is_token(context.byte) || context.byte == b' ' || context.byte == b'\t' {
                // do nothing
            } else {
                return Err(ParserError::Status(context.byte));
            }
        });

        callback_ignore_transition_fast!(self, context,
                                         on_status,
                                         State::PreHeaders1, pre_headers1);
    }

    // ---------------------------------------------------------------------------------------------
    // BODY STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn body(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        if context.handler.get_transfer_encoding() == TransferEncoding::Chunked {
            set_upper40!(self, 0);
            set_lower8!(self, 0);
            set_flag!(self, F_CHUNKED);

            transition!(self, context, State::ChunkSize, chunk_size);
        } else {
            self.content_type = context.handler.get_content_type();

            match self.content_type {
                ContentType::UrlEncoded => {
                    transition_fast!(self, context, State::UrlEncodedField, url_encoded_field);
                },
                _ => {
                    println!("This content type is not handled yet");
                }
            }
        }

        exit_eos!(self, context);
    }

    #[inline]
    fn content(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn chunk_size(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        stream_next!(context);

        match hex_to_byte(&[context.byte]) {
            Some(byte) => {
                if get_lower8!(self) == 10 {
                    // beyond max size
                    return Err(ParserError::ChunkSize(context.byte));
                }

                set_upper40!(self, get_upper40!(self) << 4);
                set_upper40!(self, get_upper40!(self) + byte as u64);
                set_lower8!(self, get_lower8!(self) + 1);

                transition!(self, context, State::ChunkSize, chunk_size);
            },
            None => {
                if get_lower8!(self) == 0 {
                    // no size supplied
                    return Err(ParserError::ChunkSize(context.byte));
                }

                if get_upper40!(self) == 0 {
                    callback_transition_fast!(self, context,
                                              on_chunk_size, get_upper40!(self),
                                              State::Newline2, newline2);
                } else if context.byte == b'\r' {
                    callback_transition_fast!(self, context,
                                              on_chunk_size, get_upper40!(self),
                                              State::ChunkSizeNewline, chunk_size_newline);
                } else if context.byte == b';' {
                    set_lower16!(self, 1);

                    callback_transition_fast!(self, context,
                                              on_chunk_size, get_upper40!(self),
                                              State::ChunkExtensionName, chunk_extension_name);
                } else {
                    Err(ParserError::ChunkSize(context.byte))
                }
            }
        }
    }

    #[inline]
    fn chunk_extension_name(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        stream_collect_tokens!(context, ParserError::ChunkExtensionName,
            callback_eos_expr!(self, context, on_chunk_extension_name),

            // stop on these bytes
            context.byte == b'='
        );

        callback_ignore_transition_fast!(self, context,
                                         on_chunk_extension_name,
                                         State::ChunkExtensionValue, chunk_extension_value);
    }

    #[inline]
    fn chunk_extension_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        stream_collect_tokens!(context, ParserError::ChunkExtensionValue,
            callback_eos_expr!(self, context, on_chunk_extension_value),

            // stop on these bytes
               context.byte == b'\r'
            || context.byte == b';'
            || context.byte == b'"'
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
        stream_next!(context);

        if is_visible!(context.byte) || context.byte == b' ' {
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
        stream_next!(context);

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
        stream_next!(context);

        if context.byte == b'\n' {
            transition!(self, context, State::ChunkData, chunk_data);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn chunk_data(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
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
        stream_next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, State::ChunkDataNewline2, chunk_data_newline2);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn chunk_data_newline2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        stream_next!(context);

        if context.byte == b'\n' {
            transition_fast!(self, context, State::ChunkSize, chunk_size);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn multipart_hyphen1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn multipart_hyphen2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn multipart_boundary(&mut self, context: &mut ParserContext<T>)
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
        stream_collect_visible!(context, ParserError::UrlEncodedField,
            callback_eos_expr!(self, context, on_url_encoded_field),

            // stop on these bytes
               context.byte == b'='
            || context.byte == b'%'
            || context.byte == b'&'
            || context.byte == b'+'
            || context.byte == b'\r'
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
        if stream_has_bytes!(context, 2) {
            stream_jump!(context, 2);

            match hex_to_byte(stream_collected_bytes!(context)) {
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
        stream_collect_visible!(context, ParserError::UrlEncodedValue,
            callback_eos_expr!(self, context, on_url_encoded_value),

            // stop on these bytes
               context.byte == b'%'
            || context.byte == b'&'
            || context.byte == b'+'
            || context.byte == b'\r'
            || context.byte == b'='
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
        if stream_has_bytes!(context, 2) {
            stream_jump!(context, 2);

            match hex_to_byte(stream_collected_bytes!(context)) {
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
    // FINISHED STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn finished_newline1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        stream_next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, State::FinishedNewline2, finished_newline2);
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn finished_newline2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        stream_next!(context);

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

// -------------------------------------------------------------------------------------------------

/// Success response types.
#[derive(Clone,Copy,PartialEq)]
pub enum Success {
    /// Callback returned false.
    Callback(usize),

    /// Additional data expected.
    Eos(usize),

    /// Finished successfully.
    Finished(usize)
}

impl fmt::Debug for Success {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Success::Callback(length) => {
                write!(formatter, "Success::Callback({})", length)
            },
            Success::Eos(length) => {
                write!(formatter, "Success::Eos({})", length)
            },
            Success::Finished(length) => {
                write!(formatter, "Success::Finished({})", length)
            }
        }
    }
}

impl fmt::Display for Success {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Success::Callback(length) => {
                write!(formatter, "{}", length)
            },
            Success::Eos(length) => {
                write!(formatter, "{}", length)
            },
            Success::Finished(length) => {
                write!(formatter, "{}", length)
            }
        }
    }
}
