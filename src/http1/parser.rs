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

//! HTTP 1.x parser.

#![allow(dead_code)]

use byte::{ is_header_field, is_quoted_header_field, is_token };
use fsm::{ ParserValue, Success };
use http1::http_handler::HttpHandler;
use http1::parser_error::ParserError;
use http1::parser_state::{dispatch, ParserState};
use http1::parser_type::ParserType;

use byte_slice::ByteStream;

// -------------------------------------------------------------------------------------------------
// MASKS AND SHIFTS
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
// MACROS
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

/// Convert hex byte to a numeric value.
///
/// This assumes the byte is 0-9, A-F, or a-f.
macro_rules! hex_to_byte {
    ($byte:expr) => (
        if $byte > 0x2F && $byte < 0x3A {
            // digit
            $byte - b'0'
        } else if $byte > 0x40 && $byte < 0x5B {
            // upper-case
            $byte - 0x37
        } else {
            // lower-case
            $byte - 0x57
        }
    );
}

/// Increase the lower 14 bits.
macro_rules! inc_lower14 {
    ($parser:expr, $length:expr) => ({
        set_lower14!(
            $parser,
            get_lower14!($parser) as usize + $length as usize
        );
    });
}

/// Increase the upper 14 bits.
macro_rules! inc_upper14 {
    ($parser:expr, $length:expr) => ({
        set_upper14!(
            $parser,
            get_upper14!($parser) as usize + $length as usize
        );
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

// -------------------------------------------------------------------------------------------------

/// HTTP 1.x parser.
pub struct Parser<'boundary> {
    /// Bit data that stores parser state details, along with HTTP major/minor versions.
    bit_data: u32,

    /// Multipart boundary.
    boundary: Option<&'boundary [u8]>,

    /// Total byte count processed.
    byte_count: usize,

    /// Length storage.
    length: usize,

    /// Parser type.
    parser_type: ParserType,

    /// Current state.
    pub state: ParserState,
}

impl<'boundary> Parser<'boundary> {
    /// Create a new `Parser` and initialize it for head parsing.
    pub fn new() -> Parser<'boundary> {
         Parser{
            bit_data:       0,
            boundary:       None,
            byte_count:     0,
            length:         0,
            parser_type:    ParserType::Head,
            state:          ParserState::StripDetect,
        }
    }

    /// Initialize this `Parser` for chunked transfer encoding parsing.
    pub fn init_chunked(&mut self) {
        self.parser_type = ParserType::Chunked;

        self.reset();
    }

    /// Initialize this `Parser` for head parsing.
    pub fn init_head(&mut self) {
        self.parser_type = ParserType::Head;

        self.reset();
    }

    /// Initialize this `Parser` for multipart parsing.
    pub fn init_multipart(&mut self) {
        self.parser_type = ParserType::Multipart;

        self.reset();
    }

    /// Initialize this `Parser` for URL encoded parsing.
    pub fn init_url_encoded(&mut self) {
        self.parser_type = ParserType::UrlEncoded;

        self.reset();
    }

    /// Retrieve the total byte count processed since the instantiation of `Parser`.
    ///
    /// The byte count is updated when `resume()` completes. This means that if a
    /// call to `byte_count()` is executed from within a callback, it will be accurate within
    /// `stream.len()` bytes. For precise accuracy, the best time to retrieve the byte count is
    /// outside of all callbacks.
    pub fn byte_count(&self) -> usize {
        self.byte_count
    }

    /// Main parser loop.
    ///
    /// # Arguments
    ///
    /// **`handler`**
    ///
    /// The handler implementation.
    #[inline]
    pub fn parse<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<Success, ParserError> {
        loop {
            match dispatch(self, handler, context) {
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

                    return Err(error);
                }
            }
        }
    }

    /// Reset `Parser` to its initial state.
    ///
    /// After each call to `reset()`, don't forget to also set the multipart boundary, or URL
    /// encoded data length using `set_boundary()` or `set_length()`.
    pub fn reset(&mut self) {
        self.bit_data = 0;
        self.boundary = None;
        self.length   = 0;

        match self.parser_type {
            ParserType::Chunked => {
                self.state          = ParserState::ChunkLength1;
            },
            ParserType::Head => {
                self.state          = ParserState::StripDetect;
            },
            ParserType::Multipart => {
                self.state          = ParserState::MultipartHyphen1;

                // lower14 == 1 when we expect a boundary, which is only the first boundary
                set_lower14!(self, 1);
            },
            ParserType::UrlEncoded => {
                self.state          = ParserState::FirstUrlEncodedName;
            }
        }
    }

    /// Resume parsing an additional slice of data.
    ///
    /// # Arguments
    ///
    /// **`handler`**
    ///
    /// The handler implementation.
    ///
    /// **`stream`**
    ///
    /// The stream of data to be parsed.
    #[inline]
    pub fn resume<T: HttpHandler>(&mut self, handler: &mut T, mut stream: &[u8])
    -> Result<Success, ParserError> {
        if let ParserType::UrlEncoded = self.parser_type {
            if self.length < stream.len() {
                // amount of data to process is less than the stream length
                stream = &stream[0..self.length];
            }

            let mut context = ByteStream::new(stream);

            match self.parse(handler, &mut context) {
                Ok(Success::Eos(length)) => {
                    if self.length - length == 0 {
                        self.state          = ParserState::BodyFinished;

                        self.parse(handler, &mut context)
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
        } else {
            self.parse(handler, &mut ByteStream::new(stream))
        }
    }

    /// Set the multipart boundary.
    pub fn set_boundary(&mut self, boundary: &'boundary [u8]) {
        self.boundary = Some(boundary);
    }

    /// Set the URL encoded length.
    pub fn set_length(&mut self, length: usize) {
        self.length = length;
    }

    /// Retrieve the current state.
    pub fn state(&self) -> ParserState {
        self.state
    }

    // ---------------------------------------------------------------------------------------------
    // RFC RULES
    // ---------------------------------------------------------------------------------------------

    /*
    The following rules are used throughout this specification to describe basic parsing constructs.
    The US-ASCII coded character set is defined by ANSI X3.4-1986.

    OCTET   = <any 8-bit sequence of data>
    CHAR    = <any US-ASCII character (octets 0 - 127)>
    UPALPHA = <any US-ASCII uppercase letter "A".."Z">
    LOALPHA = <any US-ASCII lowercase letter "a".."z">
    ALPHA   = UPALPHA | LOALPHA
    DIGIT   = <any US-ASCII digit "0".."9">
    CTL     = <any US-ASCII control character
              (octets 0 - 31) and DEL (127)>
    CR      = <US-ASCII CR, carriage return (13)>
    LF      = <US-ASCII LF, linefeed (10)>
    SP      = <US-ASCII SP, space (32)>
    HT      = <US-ASCII HT, horizontal-tab (9)>
    <">     = <US-ASCII double-quote mark (34)>

    HTTP/1.1 defines the sequence CR LF as the end-of-line marker for all protocol elements except
    the entity-body (see appendix 19.3 for tolerant applications). The end-of-line marker within an
    entity-body is defined by its associated media type, as described in section 3.7.

    CRLF = CR LF

    HTTP/1.1 header field values can be folded onto multiple lines if the continuation line begins
    with a space or horizontal tab. All linear white space, including folding, has the same
    semantics as SP. A recipient MAY replace any linear white space with a single SP before
    interpreting the field value or forwarding the message downstream.

    LWS  = [CRLF] 1*( SP | HT )

    The TEXT rule is only used for descriptive field contents and values that are not intended to be
    interpreted by the message parser. Words of *TEXT MAY contain characters from character sets
    other than ISO- 8859-1 [22] only when encoded according to the rules of RFC 2047.

    TEXT = <any OCTET except CTLs, but including LWS>

    A CRLF is allowed in the definition of TEXT only as part of a header field continuation. It is
    expected that the folding LWS will be replaced with a single SP before interpretation of the
    TEXT value.

    Hexadecimal numeric characters are used in several protocol elements.

    HEX  = "A" | "B" | "C" | "D" | "E" | "F" | "a" | "b" | "c" | "d" | "e" | "f" | DIGIT

    Many HTTP/1.1 header field values consist of words separated by LWS or special characters. These
    special characters MUST be in a quoted string to be used within a parameter value.

    token      = 1*<any CHAR except CTLs or separators>
    separators = "(" | ")" | "<" | ">" | "@"
               | "," | ";" | ":" | "\" | <">
               | "/" | "[" | "]" | "?" | "="
               | "{" | "}" | SP | HT

    Comments can be included in some HTTP header fields by surrounding the comment text with
    parentheses. Comments are only allowed in fields containing "comment" as part of their field
    value definition. In all other fields, parentheses are considered part of the field value.

    comment = "(" *( ctext | quoted-pair | comment ) ")"
    ctext   = <any TEXT excluding "(" and ")">

    A string of text is parsed as a single word if it is quoted using double-quote marks.

    quoted-string = ( <"> *(qdtext | quoted-pair ) <"> )
    qdtext        = <any TEXT except <">>

    The backslash character ("\") MAY be used as a single-character quoting mechanism only within
    quoted-string and comment constructs.

    quoted-pair = "\" CHAR
    */

    // ---------------------------------------------------------------------------------------------
    // DETECTION STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    pub fn strip_detect<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        bs_available!(context) > 0 || exit_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' || context.byte == b'\n'
        || context.byte == b'\t' || context.byte == b' ' {
            loop {
                bs_available!(context) > 0 || exit_eos!(self, context);

                bs_next!(context);

                if context.byte == b'\r' || context.byte == b'\n'
                || context.byte == b'\t' || context.byte == b' ' {
                    continue;
                }

                break;
            }
        }

        bs_replay!(context);

        transition!(
            self,
            context,
            Detect1
        );
    }

    #[inline]
    pub fn detect1<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        macro_rules! method {
            ($method:expr, $length:expr) => (
                bs_jump!(context, $length);

                callback_transition!(
                    self,
                    handler,
                    context,
                    on_method,
                    $method,
                    RequestUrl1
                );
            );
        }

        macro_rules! version {
            ($major:expr, $minor:expr, $length:expr) => ({
                bs_jump!(context, $length);
                set_state!(
                    self,
                    ResponseStatusCode1
                );

                if handler.on_version($major, $minor) {
                    transition!(self, context);
                }

                exit_callback!(self, context);
            });
        }

        unsafe {
            if bs_starts_with1!(context, b"H") {
                if bs_has_bytes!(context, 9) {
                    if bs_starts_with9!(context, b"HTTP/1.1 ") {
                        version!(1, 1, 9);
                    } else if bs_starts_with9!(context, b"HTTP/2.0 ") {
                        version!(2, 0, 9);
                    } else if bs_starts_with9!(context, b"HTTP/1.0 ") {
                        version!(1, 0, 9);
                    } else if bs_starts_with5!(context, b"HTTP/") {
                        bs_jump!(context, 5);

                        transition!(
                            self,
                            context,
                            ResponseVersionMajor1
                        );
                    }
                } else {
                    bs_jump!(context, 1);

                    transition!(
                        self,
                        context,
                        Detect2
                    );
                }
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
                } else if bs_starts_with4!(context, b"PRI ") {
                    method!(b"PRI", 4);
                }
            }
        }

        exit_if_eos!(self, context);
        bs_next!(context);

        // make sure we have an upper-cased alphabetical character
        if context.byte > 0x40 && context.byte < 0x5B {
            // this is a request
            transition_no_remark!(
                self,
                context,
                RequestMethod
            );
        }

        Err(ParserError::Method(context.byte))
    }

    #[inline]
    pub fn detect2<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'T' {
            transition!(
                self,
                context,
                Detect3
            );
        }

        // make sure we have an upper-cased alphabetical character
        if context.byte > 0x40 && context.byte < 0x5B {
            transition_no_remark!(
                self,
                context,
                RequestMethod
            );
        }

        Err(ParserError::Method(context.byte))
    }

    #[inline]
    pub fn detect3<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'T' {
            transition!(
                self,
                context,
                Detect4
            );
        }

        // make sure we have an upper-cased alphabetical character
        if context.byte > 0x40 && context.byte < 0x5B {
            transition_no_remark!(
                self,
                context,
                RequestMethod
            );
        }

        Err(ParserError::Method(context.byte))
    }

    #[inline]
    pub fn detect4<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'P' {
            transition!(
                self,
                context,
                Detect5
            );
        }

        // make sure we have an upper-cased alphabetical character
        if context.byte > 0x40 && context.byte < 0x5B {
            transition_no_remark!(
                self,
                context,
                RequestMethod
            );
        }

        Err(ParserError::Method(context.byte))
    }

    #[inline]
    pub fn detect5<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'/' {
            transition!(
                self,
                context,
                ResponseVersionMajor1
            );
        }

        // make sure we have an upper-cased alphabetical character
        if context.byte > 0x40 && context.byte < 0x5B {
            transition_no_remark!(
                self,
                context,
                RequestMethod
            );
        }

        Err(ParserError::Method(context.byte))
    }

    // ---------------------------------------------------------------------------------------------
    // REQUEST STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    pub fn request_method<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        bs_collect!(
            context,

            // break when non-upper-case is found
            if context.byte < 0x41 || context.byte > 0x5A {
                break;
            },

            // on end-of-stream
            callback_eos_expr!(self, handler, context, on_method)
        );

        if context.byte == b' ' {
            callback_ignore_transition!(
                self,
                handler,
                context,
                on_method,
                RequestUrl1
            );
        }

        Err(ParserError::Method(context.byte))
    }

    #[inline]
    pub fn request_url1<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_visible_7bit!(context.byte) {
            transition_no_remark!(
                self,
                context,
                RequestUrl2
            );
        }

        Err(ParserError::Url(context.byte))
    }

    #[inline]
    pub fn request_url2<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        collect_visible_7bit!(
            context,

            // on end-of-stream
            callback_eos_expr!(self, handler, context, on_url)
        );

        if context.byte == b' ' {
            callback_ignore_transition!(
                self,
                handler,
                context,
                on_url,
                RequestHttp1
            );
        }

        Err(ParserError::Url(context.byte))
    }

    #[inline]
    pub fn request_http1<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        macro_rules! version {
            ($major:expr, $minor:expr, $length:expr) => (
                bs_jump!(context, $length);
                set_state!(self, InitialEnd);

                if handler.on_version($major, $minor) {
                    transition!(self, context);
                }

                exit_callback!(self, context);
            );
        }

        if bs_has_bytes!(context, 9) {
            // have enough bytes to compare all known versions immediately, without collecting
            // individual tokens
            unsafe {
                if bs_starts_with9!(context, b"HTTP/1.1\r") {
                    version!(1, 1, 9);
                } else if bs_starts_with9!(context, b"HTTP/2.0\r") {
                    version!(2, 0, 9);
                } else if bs_starts_with9!(context, b"HTTP/1.0\r") {
                    version!(1, 0, 9);
                }
            }
        }

        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'H' {
            transition!(
                self,
                context,
                RequestHttp2
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn request_http2<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'T' {
            transition!(
                self,
                context,
                RequestHttp3
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn request_http3<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'T' {
            transition!(
                self,
                context,
                RequestHttp4
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn request_http4<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'P' {
            transition!(
                self,
                context,
                RequestHttp5
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn request_http5<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'/' {
            transition!(
                self,
                context,
                RequestVersionMajor1
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn request_version_major1<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        set_lower14!(self, 0);
        set_upper14!(self, 0);

        if is_digit!(context.byte) {
            set_lower14!(self, (context.byte - b'0') as u32);

            transition!(
                self,
                context,
                RequestVersionMajor2
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn request_version_major2<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'.' {
            transition!(
                self,
                context,
                RequestVersionMinor1
            );
        } else if is_digit!(context.byte) {
            set_lower14!(self, (get_lower14!(self) * 10) + (context.byte - b'0') as u32);

            transition!(
                self,
                context,
                RequestVersionMajor3
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn request_version_major3<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'.' {
            transition!(
                self,
                context,
                RequestVersionMinor1
            );
        } else if is_digit!(context.byte) {
            set_lower14!(self, (get_lower14!(self) * 10) + (context.byte - b'0') as u32);

            transition!(
                self,
                context,
                RequestVersionPeriod
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn request_version_period<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'.' {
            transition!(
                self,
                context,
                RequestVersionMinor1
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn request_version_minor1<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_digit!(context.byte) {
            set_upper14!(self, (context.byte - b'0') as u32);

            transition!(
                self,
                context,
                RequestVersionMinor2
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn request_version_minor2<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            bs_replay!(context);

            transition!(
                self,
                context,
                RequestVersionCr
            );
        } else if is_digit!(context.byte) {
            set_upper14!(self, (get_upper14!(self) * 10) + (context.byte - b'0') as u32);

            transition!(
                self,
                context,
                RequestVersionMinor3
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn request_version_minor3<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            bs_replay!(context);

            transition!(
                self,
                context,
                RequestVersionCr
            );
        } else if is_digit!(context.byte) {
            set_upper14!(self, (get_upper14!(self) * 10) + (context.byte - b'0') as u32);

            transition!(
                self,
                context,
                RequestVersionCr
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn request_version_cr<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            set_state!(self, InitialEnd);

            if handler.on_version(get_lower14!(self) as u16, get_upper14!(self) as u16) {
                transition!(
                    self,
                    context
                );
            }

            exit_callback!(self, context);
        }

        Err(ParserError::Version(context.byte))
    }

    // ---------------------------------------------------------------------------------------------
    // RESPONSE STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    pub fn response_version_major1<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        set_lower14!(self, 0);
        set_upper14!(self, 0);

        if is_digit!(context.byte) {
            set_lower14!(self, (context.byte - b'0') as u32);

            transition!(
                self,
                context,
                ResponseVersionMajor2
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn response_version_major2<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'.' {
            transition!(
                self,
                context,
                ResponseVersionMinor1
            );
        } else if is_digit!(context.byte) {
            set_lower14!(self, (get_lower14!(self) * 10) + (context.byte - b'0') as u32);

            transition!(
                self,
                context,
                ResponseVersionMajor3
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn response_version_major3<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'.' {
            transition!(
                self,
                context,
                ResponseVersionMinor1
            );
        } else if is_digit!(context.byte) {
            set_lower14!(self, (get_lower14!(self) * 10) + (context.byte - b'0') as u32);

            transition!(
                self,
                context,
                ResponseVersionPeriod
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn response_version_period<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'.' {
            transition!(
                self,
                context,
                ResponseVersionMinor1
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn response_version_minor1<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_digit!(context.byte) {
            set_upper14!(self, (context.byte - b'0') as u32);

            transition!(
                self,
                context,
                ResponseVersionMinor2
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn response_version_minor2<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b' ' {
            bs_replay!(context);

            transition!(
                self,
                context,
                ResponseVersionSpace
            );
        } else if is_digit!(context.byte) {
            set_upper14!(self, (get_upper14!(self) * 10) + (context.byte - b'0') as u32);

            transition!(
                self,
                context,
                ResponseVersionMinor3
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn response_version_minor3<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b' ' {
            bs_replay!(context);

            transition!(
                self,
                context,
                ResponseVersionSpace
            );
        } else if is_digit!(context.byte) {
            set_upper14!(self, (get_upper14!(self) * 10) + (context.byte - b'0') as u32);

            transition!(
                self,
                context,
                ResponseVersionSpace
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn response_version_space<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b' ' {
            set_state!(
                self,
                ResponseStatusCode1
            );

            if handler.on_version(get_lower14!(self) as u16, get_upper14!(self) as u16) {
                transition!(
                    self,
                    context
                );
            }

            exit_callback!(self, context);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    pub fn response_status_code1<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) > 2 {
            // have enough bytes to compare the entire status code
            bs_next!(context);

            if !is_digit!(context.byte) {
                exit_error!(StatusCode, context.byte);
            }

            set_lower14!(self, (context.byte - b'0') as u32 * 100);

            bs_next!(context);

            if !is_digit!(context.byte) {
                exit_error!(StatusCode, context.byte);
            }

            set_lower14!(self, get_lower14!(self) + (context.byte - b'0') as u32 * 10);

            bs_next!(context);

            if !is_digit!(context.byte) {
                exit_error!(StatusCode, context.byte);
            }

            set_lower14!(self, get_lower14!(self) + (context.byte - b'0') as u32);

            transition!(
                self,
                context,
                ResponseStatusCodeSpace
            );
        }

        bs_next!(context);

        if is_digit!(context.byte) {
            set_lower14!(self, (context.byte - b'0') as u32 * 100);

            transition!(
                self,
                context,
                ResponseStatusCode2
            );
        }

        Err(ParserError::StatusCode(context.byte))
    }

    #[inline]
    pub fn response_status_code2<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_digit!(context.byte) {
            set_lower14!(self, get_lower14!(self) + (context.byte - b'0') as u32 * 10);

            transition!(
                self,
                context,
                ResponseStatusCode3
            );
        }

        Err(ParserError::StatusCode(context.byte))
    }

    #[inline]
    pub fn response_status_code3<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_digit!(context.byte) {
            set_lower14!(self, get_lower14!(self) + (context.byte - b'0') as u32);

            transition!(
                self,
                context,
                ResponseStatusCodeSpace
            );
        }

        Err(ParserError::StatusCode(context.byte))
    }

    #[inline]
    pub fn response_status_code_space<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b' ' {
            set_state!(
                self,
                ResponseStatus1
            );

            if handler.on_status_code(get_lower14!(self) as u16) {
                transition!(
                    self,
                    context
                );
            }

            exit_callback!(self, context);
        }

        Err(ParserError::StatusCode(context.byte))
    }

    #[inline]
    pub fn response_status1<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        // status is any 8bit non-control byte
        if context.byte > 0x1F && context.byte != 0x7F {
            bs_replay!(context);

            transition!(
                self,
                context,
                ResponseStatus2
            );
        }

        Err(ParserError::Status(context.byte))
    }

    #[inline]
    pub fn response_status2<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        bs_collect!(
            context,

            // collect loop
            if context.byte > 0x1F && context.byte != 0x7F {
                continue;
            } else if context.byte == b'\r' {
                break;
            } else {
                exit_error!(Status, context.byte);
            },

            // on end-of-stream
            callback!(self, handler, context, on_status, {
                exit_eos!(self, context);
            })
        );

        callback_ignore_transition!(
            self,
            handler,
            context,
            on_status,
            InitialEnd
        );
    }

    // ---------------------------------------------------------------------------------------------
    // HEADER STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    pub fn initial_end<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        set_state!(self, InitialLf);

        if handler.on_initial_finished() {
            transition!(self, context);
        }

        exit_callback!(self, context);
    }

    #[inline]
    pub fn initial_lf<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition!(
                self,
                context,
                HeaderCr2
            );
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    pub fn check_header_name<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b' ' || context.byte == b'\t' {
            // multiline value
            transition!(
                self,
                context,
                HeaderValue
            );
        }

        bs_replay!(context);

        transition!(
            self,
            context,
            FirstHeaderName
        );
    }

    #[inline]
    pub fn first_header_name<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        macro_rules! name {
            ($header:expr, $length:expr) => ({
                bs_jump!(context, $length);

                callback_transition!(
                    self,
                    handler,
                    context,
                    on_header_name,
                    $header,
                    StripHeaderValue
                );
            });
        }

        if bs_has_bytes!(context, 24) {
            // have enough bytes to compare common header names immediately
            unsafe {
                if context.byte == b'A' {
                    if bs_starts_with7!(context, b"Accept:") {
                        name!(b"accept", 7);
                    } else if bs_starts_with15!(context, b"Accept-Charset:") {
                        name!(b"accept-charset", 15);
                    } else if bs_starts_with16!(context, b"Accept-Encoding:") {
                        name!(b"accept-encoding", 16);
                    } else if bs_starts_with16!(context, b"Accept-Language:") {
                        name!(b"accept-language", 16);
                    } else if bs_starts_with14!(context, b"Authorization:") {
                        name!(b"authorization", 14);
                    }
                } else if bs_starts_with5!(context, b"Host:") {
                    name!(b"host", 5);
                } else if context.byte == b'U' {
                    if bs_starts_with11!(context, b"User-Agent:") {
                        name!(b"user-agent", 11);
                    } else if bs_starts_with8!(context, b"Upgrade:") {
                        name!(b"upgrade", 8);
                    }
                } else if context.byte == b'C' {
                    if bs_starts_with11!(context, b"Connection:") {
                        name!(b"connection", 11);
                    } else if bs_starts_with13!(context, b"Content-Type:") {
                        name!(b"content-type", 13);
                    } else if bs_starts_with15!(context, b"Content-Length:") {
                        name!(b"content-length", 15);
                    } else if bs_starts_with7!(context, b"Cookie:") {
                        name!(b"cookie", 7);
                    } else if bs_starts_with14!(context, b"Cache-Control:") {
                        name!(b"cache-control", 14);
                    } else if bs_starts_with24!(context, b"Content-Security-Policy:") {
                        name!(b"content-security-policy", 24);
                    }
                } else if context.byte == b'L' {
                    if bs_starts_with9!(context, b"Location:") {
                        name!(b"location", 9);
                    } else if bs_starts_with14!(context, b"Last-Modified:") {
                        name!(b"last-modified", 14);
                    }
                } else if bs_starts_with7!(context, b"Pragma:") {
                    name!(b"pragma", 7);
                } else if bs_starts_with11!(context, b"Set-Cookie:") {
                    name!(b"set-cookie", 11);
                } else if bs_starts_with18!(context, b"Transfer-Encoding:") {
                    name!(b"transfer-encoding", 18);
                } else if context.byte == b'X' {
                    if bs_starts_with13!(context, b"X-Powered-By:") {
                        name!(b"x-powered-by", 13);
                    } else if bs_starts_with16!(context, b"X-Forwarded-For:") {
                        name!(b"x-forwarded-for", 16);
                    } else if bs_starts_with17!(context, b"X-Forwarded-Host:") {
                        name!(b"x-forwarded-host", 17);
                    } else if bs_starts_with17!(context, b"X-XSS-Protection:") {
                        name!(b"x-xss-protection", 17);
                    } else if bs_starts_with13!(context, b"X-WebKit-CSP:") {
                        name!(b"x-webkit-csp", 13);
                    }
                } else if bs_starts_with17!(context, b"WWW-Authenticate:") {
                    name!(b"www-authenticate", 17);
                }
            }
        }

        exit_if_eos!(self, context);
        bs_next!(context);

        // make sure we have something visible to collect in next state
        if is_visible_7bit!(context.byte) {
            bs_replay!(context);

            transition!(
                self,
                context,
                UpperHeaderName
            );
        }

        Err(ParserError::HeaderName(context.byte))
    }

    #[inline]
    pub fn upper_header_name<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte > 0x40 && context.byte < 0x5B {
            // upper-cased byte, let's lower-case it
            callback_transition!(
                self,
                handler,
                context,
                on_header_name,
                &[context.byte + 0x20],
                LowerHeaderName
            );
        }

        bs_replay!(context);

        transition!(
            self,
            context,
            LowerHeaderName
        );
    }

    #[inline]
    pub fn lower_header_name<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        collect_tokens!(
            context,

            // stop on these bytes
            context.byte > 0x40 && context.byte < 0x5B,

            // on end-of-stream
            callback_eos_expr!(self, handler, context, on_header_name)
        );

        if context.byte > 0x40 && context.byte < 0x5B {
            // upper-cased byte
            bs_replay!(context);

            callback_transition!(
                self,
                handler,
                context,
                on_header_name,
                UpperHeaderName
            );
        } else if context.byte == b':' {
            callback_ignore_transition!(
                self,
                handler,
                context,
                on_header_name,
                StripHeaderValue
            );
        }

        Err(ParserError::HeaderName(context.byte))
    }

    #[inline]
    pub fn strip_header_value<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(
            context,

            // on end-of-stream
            exit_eos!(self, context)
        );

        // make sure we have something visible to collect in next state
        if is_visible_7bit!(context.byte) {
            bs_replay!(context);

            transition!(
                self,
                context,
                HeaderValue
            );
        }

        Err(ParserError::HeaderValue(context.byte))
    }

    #[inline]
    pub fn header_value<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        collect_field!(
            context,

            // on end-of-stream
            callback_eos_expr!(self, handler, context, on_header_value)
        );

        if context.byte == b'\r' {
            callback_ignore_transition!(
                self,
                handler,
                context,
                on_header_value,
                HeaderLf1
            );
        } else if context.byte == b'"' {
            transition_no_remark!(
                self,
                context,
                HeaderQuotedValue
            );
        }

        Err(ParserError::HeaderValue(context.byte))
    }

    #[inline]
    pub fn header_quoted_value<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        collect_quoted_field!(
            context,

            // on end-of-stream
            callback_eos_expr!(self, handler, context, on_header_value)
        );

        if context.byte == b'"' {
            transition_no_remark!(
                self,
                context,
                HeaderValue
            );
        } else if context.byte == b'\\' {
            transition_no_remark!(
                self,
                context,
                HeaderEscapedValue
            );
        }

        Err(ParserError::HeaderValue(context.byte))
    }

    #[inline]
    pub fn header_escaped_value<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        // since we're not collecting, and because it's EOS, we must execute the callback
        // manually
        if bs_available!(context) == 0 {
            callback_eos_expr!(self, handler, context, on_header_value);
        }

        bs_next!(context);

        // escaped bytes must be 7bit, and cannot be control characters
        if context.byte > 0x1F && context.byte < 0x7B {
            callback_transition!(
                self,
                handler,
                context,
                on_header_value,
                HeaderQuotedValue
            );
        }

        Err(ParserError::HeaderValue(context.byte))
    }

    #[inline]
    pub fn header_cr1<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        unsafe {
            if bs_has_bytes!(context, 2) && bs_starts_with2!(context, b"\r\n") {
                bs_jump!(context, 2);

                transition!(
                    self,
                    context,
                    HeaderCr2
                );
            }
        }

        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition!(
                self,
                context,
                HeaderLf1
            );
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    pub fn header_lf1<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition!(
                self,
                context,
                HeaderCr2
            );
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    pub fn header_cr2<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        unsafe {
            if bs_has_bytes!(context, 2) && bs_starts_with2!(context, b"\r\n") {
                bs_jump!(context, 2);

                transition!(
                    self,
                    context,
                    HeaderEnd
                );
            }
        }

        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition!(
                self,
                context,
                HeaderLf2
            );
        }

        bs_replay!(context);

        transition!(
            self,
            context,
            CheckHeaderName
        );
    }

    #[inline]
    pub fn header_lf2<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition!(
                self,
                context,
                HeaderEnd
            );
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    pub fn header_end<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        if let ParserType::Chunked = self.parser_type {
            set_state!(self, BodyFinished);
        } else if let ParserType::Multipart = self.parser_type {
            set_state!(self, MultipartDetectData);
        } else {
            set_state!(self, Finished);
        }

        if handler.on_headers_finished() {
            transition!(self, context);
        }

        exit_callback!(self, context);
    }

    // ---------------------------------------------------------------------------------------------
    // CHUNK STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    pub fn chunk_length1<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'0' {
            set_state!(self, ChunkLengthCr);
        } else if is_hex!(context.byte) {
            self.length = hex_to_byte!(context.byte) as usize;

            set_state!(self, ChunkLength2);
        } else {
            exit_error!(ChunkLength, context.byte);
        }

        if handler.on_chunk_begin() {
            transition!(self, context);
        }

        exit_callback!(self, context);
    }

    #[inline]
    pub fn chunk_length2<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_hex!(context.byte) {
            self.length <<= 4;
            self.length  |= hex_to_byte!(context.byte) as usize;

            transition!(
                self,
                context,
                ChunkLength3
            );
        }

        bs_replay!(context);

        transition!(
            self,
            context,
            ChunkLengthCr
        );
    }

    #[inline]
    pub fn chunk_length3<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_hex!(context.byte) {
            self.length <<= 4;
            self.length  |= hex_to_byte!(context.byte) as usize;

            transition!(
                self,
                context,
                ChunkLength4
            );
        }

        bs_replay!(context);

        transition!(
            self,
            context,
            ChunkLengthCr
        );
    }

    #[inline]
    pub fn chunk_length4<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_hex!(context.byte) {
            self.length <<= 4;
            self.length  |= hex_to_byte!(context.byte) as usize;

            transition!(
                self,
                context,
                ChunkLength5
            );
        }

        bs_replay!(context);

        transition!(
            self,
            context,
            ChunkLengthCr
        );
    }

    #[inline]
    pub fn chunk_length5<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_hex!(context.byte) {
            self.length <<= 4;
            self.length  |= hex_to_byte!(context.byte) as usize;

            transition!(
                self,
                context,
                ChunkLength6
            );
        }

        bs_replay!(context);

        transition!(
            self,
            context,
            ChunkLengthCr
        );
    }

    #[inline]
    pub fn chunk_length6<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_hex!(context.byte) {
            self.length <<= 4;
            self.length  |= hex_to_byte!(context.byte) as usize;

            transition!(
                self,
                context,
                ChunkLength7
            );
        }

        bs_replay!(context);

        transition!(
            self,
            context,
            ChunkLengthCr
        );
    }

    #[inline]
    pub fn chunk_length7<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_hex!(context.byte) {
            self.length <<= 4;
            self.length  |= hex_to_byte!(context.byte) as usize;

            transition!(
                self,
                context,
                ChunkLength8
            );
        }

        bs_replay!(context);

        transition!(
            self,
            context,
            ChunkLengthCr
        );
    }

    #[inline]
    pub fn chunk_length8<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_hex!(context.byte) {
            self.length <<= 4;
            self.length  |= hex_to_byte!(context.byte) as usize;
        } else {
            bs_replay!(context);
        }

        transition!(
            self,
            context,
            ChunkLengthCr
        );
    }

    #[inline]
    pub fn chunk_length_cr<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            if self.length == 0 {
                callback_transition!(
                    self,
                    handler,
                    context,
                    on_chunk_length,
                    self.length,
                    HeaderLf1
                );
            }

            callback_transition!(
                self,
                handler,
                context,
                on_chunk_length,
                self.length,
                ChunkExtensionsFinished
            );
        } else if context.byte == b';' {
            callback_transition!(
                self,
                handler,
                context,
                on_chunk_length,
                self.length,
                StripChunkExtensionName
            );
        }

        Err(ParserError::ChunkLength(context.byte))
    }

    #[inline]
    pub fn chunk_length_lf<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition!(
                self,
                context,
                ChunkData
            );
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    pub fn strip_chunk_extension_name<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(
            context,

            // on end-of-stream
            exit_eos!(self, context)
        );

        // make sure we have something visible to collect in next state
        if is_visible_7bit!(context.byte) {
            bs_replay!(context);

            transition!(
                self,
                context,
                LowerChunkExtensionName
            );
        }

        Err(ParserError::ChunkExtensionName(context.byte))
    }

    #[inline]
    pub fn lower_chunk_extension_name<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        collect_tokens!(
            context,

            // stop on these bytes
            context.byte > 0x40 && context.byte < 0x5B,

            // on end-of-stream
            callback_eos_expr!(self, handler, context, on_chunk_extension_name)
        );

        if context.byte == b'=' {
            callback_ignore_transition!(
                self,
                handler,
                context,
                on_chunk_extension_name,
                StripChunkExtensionValue
            );
        } else if context.byte == b'\r' || context.byte == b';' {
            // extension name without a value
            bs_replay!(context);

            callback_transition!(
                self,
                handler,
                context,
                on_chunk_extension_name,
                ChunkExtensionFinished
            );
        } else if context.byte > 0x40 && context.byte < 0x5B {
            // upper-cased byte
            bs_replay!(context);

            callback_transition!(
                self,
                handler,
                context,
                on_chunk_extension_name,
                UpperChunkExtensionName
            );
        }

        Err(ParserError::ChunkExtensionName(context.byte))
    }

    #[inline]
    pub fn upper_chunk_extension_name<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte > 0x40 && context.byte < 0x5B {
            callback_transition!(
                self,
                handler,
                context,
                on_chunk_extension_name,
                &[context.byte + 0x20],
                LowerChunkExtensionName
            );
        }

        bs_replay!(context);

        transition!(
            self,
            context,
            LowerChunkExtensionName
        );
    }

    #[inline]
    pub fn strip_chunk_extension_value<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(
            context,

            // on end-of-stream
            exit_eos!(self, context)
        );

        // make sure we have something visible to collect in next state
        if is_visible_7bit!(context.byte) {
            bs_replay!(context);

            transition!(
                self,
                context,
                ChunkExtensionValue
            );
        }

        Err(ParserError::ChunkExtensionValue(context.byte))
    }

    #[inline]
    pub fn chunk_extension_value<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        collect_field!(
            context,

            // stop on these bytes
            context.byte == b';',

            // on end-of-stream
            callback_eos_expr!(self, handler, context, on_chunk_extension_value)
        );

        if context.byte == b'\r' || context.byte == b';' {
            bs_replay!(context);

            callback_transition!(
                self,
                handler,
                context,
                on_chunk_extension_value,
                ChunkExtensionFinished
            );
        } else if context.byte == b'"' {
            callback_ignore_transition!(
                self,
                handler,
                context,
                on_chunk_extension_value,
                ChunkExtensionQuotedValue
            );
        }

        Err(ParserError::ChunkExtensionValue(context.byte))
    }

    #[inline]
    pub fn chunk_extension_quoted_value<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        collect_quoted_field!(
            context,

            // on end-of-stream
            callback_eos_expr!(self, handler, context, on_chunk_extension_value)
        );

        if context.byte == b'"' {
            callback_ignore_transition!(
                self,
                handler,
                context,
                on_chunk_extension_value,
                ChunkExtensionValue
            );
        } else if context.byte == b'\\' {
            callback_ignore_transition!(
                self,
                handler,
                context,
                on_chunk_extension_value,
                ChunkExtensionEscapedValue
            );
        }

        Err(ParserError::ChunkExtensionValue(context.byte))
    }

    #[inline]
    pub fn chunk_extension_escaped_value<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        // escaped bytes must be 7bit, and cannot be control characters
        if context.byte > 0x1F && context.byte < 0x7B {
            callback_transition!(
                self,
                handler,
                context,
                on_chunk_extension_value,
                &[context.byte],
                ChunkExtensionQuotedValue
            );
        }

        Err(ParserError::ChunkExtensionValue(context.byte))
    }

    #[inline]
    pub fn chunk_extension_finished<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            set_state!(self, ChunkExtensionsFinished);
        } else {
            set_state!(self, StripChunkExtensionName);
        }

        if handler.on_chunk_extension_finished() {
            transition!(self, context);
        }

        exit_callback!(self, context);
    }

    #[inline]
    pub fn chunk_extensions_finished<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        set_state!(self, ChunkLengthLf);

        if handler.on_chunk_extensions_finished() {
            transition!(
                self,
                context
            );
        }

        exit_callback!(self, context);
    }

    #[inline]
    pub fn chunk_data<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= self.length {
            // collect remaining chunk data
            bs_collect_length!(context, self.length);

            self.length = 0;

            callback_transition!(
                self,
                handler,
                context,
                on_chunk_data,
                ChunkDataCr
            );
        }

        // collect remaining stream data
        self.length -= bs_available!(context);

        bs_collect_length!(context, bs_available!(context));

        callback_transition!(
            self,
            handler,
            context,
            on_chunk_data,
            ChunkData
        );
    }

    #[inline]
    pub fn chunk_data_cr<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition!(
                self,
                context,
                ChunkDataLf
            );
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    pub fn chunk_data_lf<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition!(
                self,
                context,
                ChunkLength1
            );
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    // ---------------------------------------------------------------------------------------------
    // MULTIPART STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    pub fn multipart_hyphen1<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'-' {
            transition!(
                self,
                context,
                MultipartHyphen2
            );
        } else if get_lower14!(self) == 0 {
            // we're checking for the boundary within multipart data, but it's not the boundary,
            // so let's send the data to the callback and get back to parsing
            callback_transition!(
                self,
                handler,
                context,
                on_multipart_data,
                &[b'\r', b'\n', context.byte],
                MultipartDataByByte
            );
        }

        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    pub fn multipart_hyphen2<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'-' {
            transition!(
                self,
                context,
                MultipartBoundary
            );
        } else if get_lower14!(self) == 0 {
            // we're checking for the boundary within multipart data, but it's not the boundary,
            // so let's send the data to the callback and get back to parsing
            callback_transition!(
                self,
                handler,
                context,
                on_multipart_data,
                &[b'\r', b'\n', b'-', context.byte],
                MultipartDataByByte
            );
        }

        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    pub fn multipart_boundary<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        let (length, callback_data, finished) = {
            let boundary = self.boundary.unwrap();

            let slice =
                if boundary.len() - get_upper14!(self) as usize <= bs_available!(context) {
                    // compare remainder of boundary
                    &boundary[get_upper14!(self) as usize..]
                } else {
                    // compare remainder of stream
                    &boundary[
                        get_upper14!(self) as usize..
                        get_upper14!(self) as usize + bs_available!(context)
                    ]
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

                callback_transition!(
                    self,
                    handler,
                    context,
                    on_multipart_data,
                    &v,
                    MultipartDataByByte
                );
            }

            // we're parsing the initial boundary, and it's invalid
            //
            // there is one caveat to this error:
            //     it will always report the first byte being invalid, even if
            //     it's another byte that did not match, because we're using
            //     bs_starts_with!() vs an individual byte check
            bs_next!(context);

            exit_error!(MultipartBoundary, context.byte);
        } else if finished {
            // boundary comparison finished

            // reset boundary comparison index
            set_upper14!(self, 0);

            transition!(
                self,
                context,
                MultipartBoundaryCr
            );
        }

        // boundary comparison not finished
        inc_upper14!(self, length);

        exit_eos!(self, context);
    }

    #[inline]
    pub fn multipart_boundary_cr<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            set_state!(self, InitialLf);

            if handler.on_multipart_begin() {
                transition!(self, context);
            }

            exit_callback!(self, context);
        } else if context.byte == b'-' {
            transition!(
                self,
                context,
                MultipartEnd
            );
        }

        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    pub fn multipart_boundary_lf<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition!(
                self,
                context,
                CheckHeaderName
            );
        }

        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    pub fn multipart_detect_data<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        if let Some(length) = handler.content_length() {
            self.length = length;

            // expect boundary after data
            set_lower14!(self, 1);

            transition!(
                self,
                context,
                MultipartDataByLength
            );
        }

        // do not expect boundary since it can be part of the data itself
        set_lower14!(self, 0);

        transition!(
            self,
            context,
            MultipartDataByByte
        );
    }

    #[inline]
    pub fn multipart_data_by_length<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= self.length {
            // collect remaining multipart data
            bs_collect_length!(context, self.length);

            self.length = 0;

            callback_transition!(
                self,
                handler,
                context,
                on_multipart_data,
                MultipartDataByLengthCr
            );
        }

        // collect remaining stream data
        self.length -= bs_available!(context);

        bs_collect_length!(context, bs_available!(context));

        callback_transition!(
            self,
            handler,
            context,
            on_multipart_data,
            MultipartDataByLength
        );
    }

    #[inline]
    pub fn multipart_data_by_length_cr<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition!(
                self,
                context,
                MultipartDataByLengthLf
            );
        }

        // this state is only used after multipart_data_by_length, so we can error if we don't
        // find the carriage return
        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    pub fn multipart_data_by_length_lf<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition!(
                self,
                context,
                MultipartHyphen1
            );
        }

        // this state is only used after multipart_data_by_length, so we can error if we don't
        // find the carriage return
        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    pub fn multipart_data_by_byte<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        bs_collect_until!(
            context,

            // collect bytes until
            context.byte == b'\r',

            // on end-of-stream
            callback_eos_expr!(self, handler, context, on_multipart_data)
        );

        callback_ignore_transition!(
            self,
            handler,
            context,
            on_multipart_data,
            MultipartDataByByteLf
        );
    }

    #[inline]
    pub fn multipart_data_by_byte_lf<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition!(
                self,
                context,
                MultipartHyphen1
            );
        }

        callback_transition!(
            self,
            handler,
            context,
            on_multipart_data,
            &[b'\r', context.byte],
            MultipartDataByByte
        );
    }

    #[inline]
    pub fn multipart_end<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'-' {
            transition!(
                self,
                context,
                BodyFinished
            );
        }

        Err(ParserError::MultipartBoundary(context.byte))
    }

    // ---------------------------------------------------------------------------------------------
    // URL ENCODED STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    pub fn first_url_encoded_name<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        bs_available!(context) > 0 || exit_eos!(self, context);

        set_state!(self, UrlEncodedName);

        if handler.on_url_encoded_begin() {
            transition!(
                self,
                context,
                UrlEncodedName
            )
        }

        exit_callback!(self, context);
    }

    #[inline]
    pub fn url_encoded_name<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        collect_visible_7bit!(
            context,

            // stop on these bytes
               context.byte == b'%'
            || context.byte == b'+'
            || context.byte == b'='
            || context.byte == b'&'
            || context.byte == b';',

            // on end-of-stream
            callback_eos_expr!(self, handler, context, on_url_encoded_name)
        );

        match context.byte {
            b'%' => {
                callback_ignore_transition!(
                    self,
                    handler,
                    context,
                    on_url_encoded_name,
                    UrlEncodedNameHex1
                );
            },
            b'+' => {
                callback_ignore_transition!(
                    self,
                    handler,
                    context,
                    on_url_encoded_name,
                    UrlEncodedNamePlus
                );
            },
            b'=' => {
                callback_ignore_transition!(
                    self,
                    handler,
                    context,
                    on_url_encoded_name,
                    UrlEncodedValue
                );
            },
            b'&' | b';' => {
                callback_ignore_transition!(
                    self,
                    handler,
                    context,
                    on_url_encoded_name,
                    FirstUrlEncodedName
                );
            },
            _ => {
                Err(ParserError::UrlEncodedName(context.byte))
            }
        }
    }

    #[inline]
    pub fn url_encoded_name_hex1<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        set_upper14!(
            self,
            if is_digit!(context.byte) {
                (context.byte - b'0') << 4
            } else if b'@' < context.byte && context.byte < b'G' {
                (context.byte - 0x37) << 4
            } else if b'`' < context.byte && context.byte < b'g' {
                (context.byte - 0x57) << 4
            } else {
                exit_error!(UrlEncodedName, context.byte);
            }
        );

        transition!(
            self,
            context,
            UrlEncodedNameHex2
        );
    }

    #[inline]
    pub fn url_encoded_name_hex2<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        set_lower14!(
            self,
            if is_digit!(context.byte) {
                context.byte - b'0'
            } else if b'@' < context.byte && context.byte < b'G' {
                context.byte - 0x37
            } else if b'`' < context.byte && context.byte < b'g' {
                context.byte - 0x57
            } else {
                exit_error!(UrlEncodedName, context.byte);
            }
        );

        callback_transition!(
            self,
            handler,
            context,
            on_url_encoded_name,
            &[(get_upper14!(self) | get_lower14!(self)) as u8],
            UrlEncodedName
        );
    }

    #[inline]
    pub fn url_encoded_name_plus<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        callback_transition!(
            self,
            handler,
            context,
            on_url_encoded_name,
            b" ",
            UrlEncodedName
        );
    }

    #[inline]
    pub fn url_encoded_value<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        collect_visible_7bit!(
            context,

            // stop on these bytes
               context.byte == b'%'
            || context.byte == b'+'
            || context.byte == b'&'
            || context.byte == b';',

            // on end-of-stream
            callback_eos_expr!(self, handler, context, on_url_encoded_value)
        );

        match context.byte {
            b'%' => {
                callback_ignore_transition!(
                    self,
                    handler,
                    context,
                    on_url_encoded_value,
                    UrlEncodedValueHex1
                );
            },
            b'&' | b';' => {
                callback_ignore_transition!(
                    self,
                    handler,
                    context,
                    on_url_encoded_value,
                    FirstUrlEncodedName
                );
            },
            b'+' => {
                callback_ignore_transition!(
                    self,
                    handler,
                    context,
                    on_url_encoded_value,
                    UrlEncodedValuePlus
                );
            },
            _ => {
                Err(ParserError::UrlEncodedValue(context.byte))
            }
        }
    }

    #[inline]
    pub fn url_encoded_value_hex1<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        set_upper14!(
            self,
            if is_digit!(context.byte) {
                (context.byte - b'0') << 4
            } else if b'@' < context.byte && context.byte < b'G' {
                (context.byte - 0x37) << 4
            } else if b'`' < context.byte && context.byte < b'g' {
                (context.byte - 0x57) << 4
            } else {
                exit_error!(UrlEncodedValue, context.byte);
            }
        );

        transition!(
            self,
            context,
            UrlEncodedValueHex2
        );
    }

    #[inline]
    pub fn url_encoded_value_hex2<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        set_lower14!(
            self,
            if is_digit!(context.byte) {
                context.byte - b'0'
            } else if b'@' < context.byte && context.byte < b'G' {
                context.byte - 0x37
            } else if b'`' < context.byte && context.byte < b'g' {
                context.byte - 0x57
            } else {
                exit_error!(UrlEncodedValue, context.byte);
            }
        );

        callback_transition!(
            self,
            handler,
            context,
            on_url_encoded_value,
            &[(get_upper14!(self) | get_lower14!(self)) as u8],
            UrlEncodedValue
        );
    }

    #[inline]
    pub fn url_encoded_value_plus<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        callback_transition!(
            self,
            handler,
            context,
            on_url_encoded_value,
            b" ",
            UrlEncodedValue
        );
    }

    // ---------------------------------------------------------------------------------------------
    // DEAD & FINISHED STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    pub fn dead<T: HttpHandler>(&mut self, _handler: &mut T, _context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_error!(Dead);
    }

    #[inline]
    pub fn body_finished<T: HttpHandler>(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        set_state!(self, Finished);

        if handler.on_body_finished() {
            transition!(self, context);
        }

        exit_callback!(self, context);
    }

    #[inline]
    pub fn finished<T: HttpHandler>(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_finished!(self, context);
    }
}
