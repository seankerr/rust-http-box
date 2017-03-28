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

use byte::{ is_header_field, is_quoted_header_field, is_token, is_url_encoded_separator };
use fsm::{ ParserValue, Success };
use http1::http_handler::HttpHandler;
use http1::parser_error::ParserError;
use http1::parser_state::ParserState;
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
pub struct Parser<'a, T: HttpHandler + 'a> {
    /// Bit data that stores parser state details, along with HTTP major/minor versions.
    bit_data: u32,

    /// Multipart boundary.
    boundary: Option<&'a [u8]>,

    /// Total byte count processed.
    byte_count: usize,

    /// Length storage.
    length: usize,

    /// Parser type.
    parser_type: ParserType,

    /// Current state.
    state: ParserState,

    /// Current state function.
    state_function: fn(&mut Parser<'a, T>, &mut T, &mut ByteStream)
                    -> Result<ParserValue, ParserError>
}

impl<'a, T: HttpHandler + 'a> Parser<'a, T> {
    /// Create a new `Parser` and initialize it for head parsing.
    pub fn new() -> Parser<'a, T> {
         Parser{
            bit_data:       0,
            boundary:       None,
            byte_count:     0,
            length:         0,
            parser_type:    ParserType::Head,
            state:          ParserState::StripDetect,
            state_function: Parser::detect1
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
    fn parse(&mut self, mut handler: &mut T, mut context: &mut ByteStream)
    -> Result<Success, ParserError> {
        loop {
            match (self.state_function)(self, &mut handler, &mut context) {
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
                self.state_function = Parser::chunk_length1;
            },
            ParserType::Head => {
                self.state          = ParserState::StripDetect;
                self.state_function = Parser::strip_detect;
            },
            ParserType::Multipart => {
                self.state          = ParserState::MultipartHyphen1;
                self.state_function = Parser::multipart_hyphen1;

                // lower14 == 1 when we expect a boundary, which is only the first boundary
                set_lower14!(self, 1);
            },
            ParserType::UrlEncoded => {
                self.state          = ParserState::UrlEncodedName;
                self.state_function = Parser::url_encoded_name;
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
    pub fn resume(&mut self, mut handler: &mut T, mut stream: &[u8])
    -> Result<Success, ParserError> {
        if let ParserType::UrlEncoded = self.parser_type {
            if self.length < stream.len() {
                // amount of data to process is less than the stream length
                stream = &stream[0..self.length];
            }

            let mut context = ByteStream::new(stream);

            match self.parse(&mut handler, &mut context) {
                Ok(Success::Eos(length)) => {
                    if self.length - length == 0 {
                        self.state          = ParserState::BodyFinished;
                        self.state_function = Parser::body_finished;

                        self.parse(&mut handler, &mut context)
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
            self.parse(&mut handler, &mut ByteStream::new(stream))
        }
    }

    /// Set the multipart boundary.
    pub fn set_boundary(&mut self, boundary: &'a [u8]) {
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
    fn strip_detect(&mut self, handler: &mut T, context: &mut ByteStream)
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

        transition_fast!(
            self,
            handler,
            context,
            Detect1,
            detect1
        );
    }

    #[inline]
    fn detect1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        macro_rules! method {
            ($method:expr, $length:expr) => (
                bs_jump!(context, $length);

                callback_transition_fast!(
                    self,
                    handler,
                    context,
                    on_method,
                    $method,
                    RequestUrl1,
                    request_url1
                );
            );
        }

        macro_rules! version {
            ($major:expr, $minor:expr, $length:expr) => ({
                bs_jump!(context, $length);
                set_state!(
                    self,
                    ResponseStatusCode1,
                    response_status_code1
                );

                if handler.on_version($major, $minor) {
                    transition_fast!(self, handler, context);
                }

                exit_callback!(self, context);
            });
        }

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

                    transition_fast!(
                        self,
                        handler,
                        context,
                        ResponseVersionMajor1,
                        response_version_major1
                    );
                }
            } else {
                bs_jump!(context, 1);

                transition_fast!(
                    self,
                    handler,
                    context,
                    Detect2,
                    detect2
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

        exit_if_eos!(self, context);
        bs_next!(context);

        // make sure we have an upper-cased alphabetical character
        if context.byte > 0x40 && context.byte < 0x5B {
            // this is a request
            transition_fast_no_remark!(
                self,
                handler,
                context,
                RequestMethod,
                request_method
            );
        }

        Err(ParserError::Method(context.byte))
    }

    #[inline]
    fn detect2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'T' {
            transition_fast!(
                self,
                handler,
                context,
                Detect3,
                detect3
            );
        }

        // make sure we have an upper-cased alphabetical character
        if context.byte > 0x40 && context.byte < 0x5B {
            transition_fast_no_remark!(
                self,
                handler,
                context,
                RequestMethod,
                request_method
            );
        }

        Err(ParserError::Method(context.byte))
    }

    #[inline]
    fn detect3(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'T' {
            transition_fast!(
                self,
                handler,
                context,
                Detect4,
                detect4
            );
        }

        // make sure we have an upper-cased alphabetical character
        if context.byte > 0x40 && context.byte < 0x5B {
            transition_fast_no_remark!(
                self,
                handler,
                context,
                RequestMethod,
                request_method
            );
        }

        Err(ParserError::Method(context.byte))
    }

    #[inline]
    fn detect4(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'P' {
            transition_fast!(
                self,
                handler,
                context,
                Detect5,
                detect5
            );
        }

        // make sure we have an upper-cased alphabetical character
        if context.byte > 0x40 && context.byte < 0x5B {
            transition_fast_no_remark!(
                self,
                handler,
                context,
                RequestMethod,
                request_method
            );
        }

        Err(ParserError::Method(context.byte))
    }

    #[inline]
    fn detect5(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'/' {
            transition_fast!(
                self,
                handler,
                context,
                ResponseVersionMajor1,
                response_version_major1
            );
        }

        // make sure we have an upper-cased alphabetical character
        if context.byte > 0x40 && context.byte < 0x5B {
            transition_fast_no_remark!(
                self,
                handler,
                context,
                RequestMethod,
                request_method
            );
        }

        Err(ParserError::Method(context.byte))
    }

    // ---------------------------------------------------------------------------------------------
    // REQUEST STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn request_method(&mut self, handler: &mut T, context: &mut ByteStream)
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
            callback_ignore_transition_fast!(
                self,
                handler,
                context,
                on_method,
                RequestUrl1,
                request_url1
            );
        }

        Err(ParserError::Method(context.byte))
    }

    #[inline]
    fn request_url1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_visible_7bit!(context.byte) {
            transition_fast_no_remark!(
                self,
                handler,
                context,
                RequestUrl2,
                request_url2
            );
        }

        Err(ParserError::Url(context.byte))
    }

    #[inline]
    fn request_url2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        collect_visible_7bit!(
            context,

            // on end-of-stream
            callback_eos_expr!(self, handler, context, on_url)
        );

        if context.byte == b' ' {
            callback_ignore_transition_fast!(
                self,
                handler,
                context,
                on_url,
                RequestHttp1,
                request_http1
            );
        }

        Err(ParserError::Url(context.byte))
    }

    #[inline]
    fn request_http1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        macro_rules! version {
            ($major:expr, $minor:expr, $length:expr) => (
                bs_jump!(context, $length);
                set_state!(self, InitialEnd, initial_end);

                if handler.on_version($major, $minor) {
                    transition_fast!(self, handler, context);
                }

                exit_callback!(self, context);
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

        if context.byte == b'H' {
            transition_fast!(self, handler, context, RequestHttp2, request_http2);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_http2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'T' {
            transition_fast!(
                self,
                handler,
                context,
                RequestHttp3,
                request_http3
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_http3(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'T' {
            transition_fast!(
                self,
                handler,
                context,
                RequestHttp4,
                request_http4
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_http4(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'P' {
            transition_fast!(
                self,
                handler,
                context,
                RequestHttp5,
                request_http5
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_http5(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'/' {
            transition_fast!(
                self,
                handler,
                context,
                RequestVersionMajor1,
                request_version_major1
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_version_major1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        set_lower14!(self, 0);
        set_upper14!(self, 0);

        if is_digit!(context.byte) {
            set_lower14!(self, (context.byte - b'0') as u32);

            transition_fast!(
                self,
                handler,
                context,
                RequestVersionMajor2,
                request_version_major2
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_version_major2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'.' {
            transition_fast!(
                self,
                handler,
                context,
                RequestVersionMinor1,
                request_version_minor1
            );
        } else if is_digit!(context.byte) {
            set_lower14!(self, (get_lower14!(self) * 10) + (context.byte - b'0') as u32);

            transition_fast!(
                self,
                handler,
                context,
                RequestVersionMajor3,
                request_version_major3
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_version_major3(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'.' {
            transition_fast!(
                self,
                handler,
                context,
                RequestVersionMinor1,
                request_version_minor1
            );
        } else if is_digit!(context.byte) {
            set_lower14!(self, (get_lower14!(self) * 10) + (context.byte - b'0') as u32);

            transition_fast!(
                self,
                handler,
                context,
                RequestVersionPeriod,
                request_version_period
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_version_period(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'.' {
            transition_fast!(
                self,
                handler,
                context,
                RequestVersionMinor1,
                request_version_minor1
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_version_minor1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_digit!(context.byte) {
            set_upper14!(self, (context.byte - b'0') as u32);

            transition_fast!(
                self,
                handler,
                context,
                RequestVersionMinor2,
                request_version_minor2
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_version_minor2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            bs_replay!(context);

            transition_fast!(
                self,
                handler,
                context,
                RequestVersionCr,
                request_version_cr
            );
        } else if is_digit!(context.byte) {
            set_upper14!(self, (get_upper14!(self) * 10) + (context.byte - b'0') as u32);

            transition_fast!(
                self,
                handler,
                context,
                RequestVersionMinor3,
                request_version_minor3
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_version_minor3(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            bs_replay!(context);

            transition_fast!(
                self,
                handler,
                context,
                RequestVersionCr,
                request_version_cr
            );
        } else if is_digit!(context.byte) {
            set_upper14!(self, (get_upper14!(self) * 10) + (context.byte - b'0') as u32);

            transition_fast!(
                self,
                handler,
                context,
                RequestVersionCr,
                request_version_cr
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn request_version_cr(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            set_state!(self, InitialEnd, initial_end);

            if handler.on_version(get_lower14!(self) as u16, get_upper14!(self) as u16) {
                transition_fast!(
                    self,
                    handler,
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
    fn response_version_major1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        set_lower14!(self, 0);
        set_upper14!(self, 0);

        if is_digit!(context.byte) {
            set_lower14!(self, (context.byte - b'0') as u32);

            transition_fast!(
                self,
                handler,
                context,
                ResponseVersionMajor2,
                response_version_major2
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn response_version_major2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'.' {
            transition_fast!(
                self,
                handler,
                context,
                ResponseVersionMinor1,
                response_version_minor1
            );
        } else if is_digit!(context.byte) {
            set_lower14!(self, (get_lower14!(self) * 10) + (context.byte - b'0') as u32);

            transition_fast!(
                self,
                handler,
                context,
                ResponseVersionMajor3,
                response_version_major3
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn response_version_major3(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'.' {
            transition_fast!(
                self,
                handler,
                context,
                ResponseVersionMinor1,
                response_version_minor1
            );
        } else if is_digit!(context.byte) {
            set_lower14!(self, (get_lower14!(self) * 10) + (context.byte - b'0') as u32);

            transition_fast!(
                self,
                handler,
                context,
                ResponseVersionPeriod,
                response_version_period
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn response_version_period(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'.' {
            transition_fast!(
                self,
                handler,
                context,
                ResponseVersionMinor1,
                response_version_minor1
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn response_version_minor1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_digit!(context.byte) {
            set_upper14!(self, (context.byte - b'0') as u32);

            transition_fast!(
                self,
                handler,
                context,
                ResponseVersionMinor2,
                response_version_minor2
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn response_version_minor2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b' ' {
            bs_replay!(context);

            transition_fast!(
                self,
                handler,
                context,
                ResponseVersionSpace,
                response_version_space
            );
        } else if is_digit!(context.byte) {
            set_upper14!(self, (get_upper14!(self) * 10) + (context.byte - b'0') as u32);

            transition_fast!(
                self,
                handler,
                context,
                ResponseVersionMinor3,
                response_version_minor3
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn response_version_minor3(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b' ' {
            bs_replay!(context);

            transition_fast!(
                self,
                handler,
                context,
                ResponseVersionSpace,
                response_version_space
            );
        } else if is_digit!(context.byte) {
            set_upper14!(self, (get_upper14!(self) * 10) + (context.byte - b'0') as u32);

            transition_fast!(
                self,
                handler,
                context,
                ResponseVersionSpace,
                response_version_space
            );
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn response_version_space(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b' ' {
            set_state!(
                self,
                ResponseStatusCode1,
                response_status_code1
            );

            if handler.on_version(get_lower14!(self) as u16, get_upper14!(self) as u16) {
                transition_fast!(
                    self,
                    handler,
                    context
                );
            }

            exit_callback!(self, context);
        }

        Err(ParserError::Version(context.byte))
    }

    #[inline]
    fn response_status_code1(&mut self, handler: &mut T, context: &mut ByteStream)
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

            transition_fast!(
                self,
                handler,
                context,
                ResponseStatusCodeSpace,
                response_status_code_space
            );
        }

        bs_next!(context);

        if is_digit!(context.byte) {
            set_lower14!(self, (context.byte - b'0') as u32 * 100);

            transition_fast!(
                self,
                handler,
                context,
                ResponseStatusCode2,
                response_status_code2
            );
        }

        Err(ParserError::StatusCode(context.byte))
    }

    #[inline]
    fn response_status_code2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_digit!(context.byte) {
            set_lower14!(self, get_lower14!(self) + (context.byte - b'0') as u32 * 10);

            transition_fast!(
                self,
                handler,
                context,
                ResponseStatusCode3,
                response_status_code3
            );
        }

        Err(ParserError::StatusCode(context.byte))
    }

    #[inline]
    fn response_status_code3(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_digit!(context.byte) {
            set_lower14!(self, get_lower14!(self) + (context.byte - b'0') as u32);

            transition_fast!(
                self,
                handler,
                context,
                ResponseStatusCodeSpace,
                response_status_code_space
            );
        }

        Err(ParserError::StatusCode(context.byte))
    }

    #[inline]
    fn response_status_code_space(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b' ' {
            set_state!(
                self,
                ResponseStatus1,
                response_status1
            );

            if handler.on_status_code(get_lower14!(self) as u16) {
                transition_fast!(
                    self,
                    handler,
                    context
                );
            }

            exit_callback!(self, context);
        }

        Err(ParserError::StatusCode(context.byte))
    }

    #[inline]
    fn response_status1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        // status is any 8bit non-control byte
        if context.byte > 0x1F && context.byte != 0x7F {
            bs_replay!(context);

            transition_fast!(
                self,
                handler,
                context,
                ResponseStatus2,
                response_status2
            );
        }

        Err(ParserError::Status(context.byte))
    }

    #[inline]
    fn response_status2(&mut self, handler: &mut T, context: &mut ByteStream)
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

        callback_ignore_transition_fast!(
            self,
            handler,
            context,
            on_status,
            InitialEnd,
            initial_end
        );
    }

    // ---------------------------------------------------------------------------------------------
    // HEADER STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn initial_end(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        set_state!(self, InitialLf, initial_lf);

        if handler.on_initial_finished() {
            transition_fast!(self, handler, context);
        }

        exit_callback!(self, context);
    }

    #[inline]
    fn initial_lf(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition_fast!(
                self,
                handler,
                context,
                HeaderCr2,
                header_cr2
            );
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn check_header_name(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b' ' || context.byte == b'\t' {
            // multiline value
            transition_fast!(
                self,
                handler,
                context,
                HeaderValue,
                header_value
            );
        }

        bs_replay!(context);

        transition_fast!(
            self,
            handler,
            context,
            FirstHeaderName,
            first_header_name
        );
    }

    #[inline]
    fn first_header_name(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        macro_rules! name {
            ($header:expr, $length:expr) => ({
                bs_jump!(context, $length);

                callback_transition_fast!(
                    self,
                    handler,
                    context,
                    on_header_name,
                    $header,
                    StripHeaderValue,
                    strip_header_value
                );
            });
        }

        if bs_has_bytes!(context, 24) {
            // have enough bytes to compare common header names immediately, without collecting
            // individual tokens
            if context.byte == b'C' {
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
            } else if context.byte == b'A' {
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
            } else if context.byte == b'U' {
                if bs_starts_with11!(context, b"User-Agent:") {
                    name!(b"user-agent", 11);
                } else if bs_starts_with8!(context, b"Upgrade:") {
                    name!(b"upgrade", 8);
                }
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

        exit_if_eos!(self, context);
        bs_next!(context);

        // make sure we have something visible to collect in next state
        if is_visible_7bit!(context.byte) {
            bs_replay!(context);

            transition_fast!(
                self,
                handler,
                context,
                UpperHeaderName,
                upper_header_name
            );
        }

        Err(ParserError::HeaderName(context.byte))
    }

    #[inline]
    fn upper_header_name(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte > 0x40 && context.byte < 0x5B {
            // upper-cased byte, let's lower-case it
            callback_transition_fast!(
                self,
                handler,
                context,
                on_header_name,
                &[context.byte + 0x20],
                LowerHeaderName,
                lower_header_name
            );
        }

        bs_replay!(context);

        transition_fast!(
            self,
            handler,
            context,
            LowerHeaderName,
            lower_header_name
        );
    }

    #[inline]
    fn lower_header_name(&mut self, handler: &mut T, context: &mut ByteStream)
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
                UpperHeaderName,
                upper_header_name
            );
        } else if context.byte == b':' {
            callback_ignore_transition!(
                self,
                handler,
                context,
                on_header_name,
                StripHeaderValue,
                strip_header_value
            );
        }

        Err(ParserError::HeaderName(context.byte))
    }

    #[inline]
    fn strip_header_value(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(
            context,

            // on end-of-stream
            exit_eos!(self, context)
        );

        // make sure we have something visible to collect in next state
        if is_visible_7bit!(context.byte) {
            bs_replay!(context);

            transition_fast!(
                self,
                handler,
                context,
                HeaderValue,
                header_value
            );
        }

        Err(ParserError::HeaderValue(context.byte))
    }

    #[inline]
    fn header_value(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        collect_field!(
            context,

            // on end-of-stream
            callback_eos_expr!(self, handler, context, on_header_value)
        );

        if context.byte == b'\r' {
            callback_ignore_transition_fast!(
                self,
                handler,
                context,
                on_header_value,
                HeaderLf1,
                header_lf1
            );
        } else if context.byte == b'"' {
            transition_fast_no_remark!(
                self,
                handler,
                context,
                HeaderQuotedValue,
                header_quoted_value
            );
        }

        Err(ParserError::HeaderValue(context.byte))
    }

    #[inline]
    fn header_quoted_value(&mut self, handler: &mut T, context: &mut ByteStream)
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
                HeaderValue,
                header_value
            );
        } else if context.byte == b'\\' {
            transition_fast_no_remark!(
                self,
                handler,
                context,
                HeaderEscapedValue,
                header_escaped_value
            );
        }

        Err(ParserError::HeaderValue(context.byte))
    }

    #[inline]
    fn header_escaped_value(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        // since we're not collecting, and because it's EOS, we must execute the callback
        // manually
           bs_available!(context) > 0
        || callback_eos_expr!(self, handler, context, on_header_value);

        bs_next!(context);

        // escaped bytes must be 7bit, and cannot be control characters
        if context.byte > 0x1F && context.byte < 0x7B {
            callback_transition!(
                self,
                handler,
                context,
                on_header_value,
                HeaderQuotedValue,
                header_quoted_value
            );
        }

        Err(ParserError::HeaderValue(context.byte))
    }

    #[inline]
    fn header_cr1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        if bs_has_bytes!(context, 2) && bs_starts_with2!(context, b"\r\n") {
            bs_jump!(context, 2);

            transition_fast!(
                self,
                handler,
                context,
                HeaderCr2,
                header_cr2
            );
        }

        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition_fast!(
                self,
                handler,
                context,
                HeaderLf1,
                header_lf1
            );
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn header_lf1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition_fast!(
                self,
                handler,
                context,
                HeaderCr2,
                header_cr2
            );
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn header_cr2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        if bs_has_bytes!(context, 2) && bs_starts_with2!(context, b"\r\n") {
            bs_jump!(context, 2);

            transition_fast!(
                self,
                handler,
                context,
                HeaderEnd,
                header_end
            );
        }

        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition_fast!(
                self,
                handler,
                context,
                HeaderLf2,
                header_lf2
            );
        }

        bs_replay!(context);

        transition!(
            self,
            context,
            CheckHeaderName,
            check_header_name
        );
    }

    #[inline]
    fn header_lf2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition_fast!(
                self,
                handler,
                context,
                HeaderEnd,
                header_end
            );
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn header_end(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        if let ParserType::Chunked = self.parser_type {
            set_state!(self, BodyFinished, body_finished);
        } else if let ParserType::Multipart = self.parser_type {
            set_state!(self, MultipartDetectData, multipart_detect_data);
        } else {
            set_state!(self, Finished, finished);
        }

        if handler.on_headers_finished() {
            transition_fast!(self, handler, context);
        }

        exit_callback!(self, context);
    }

    // ---------------------------------------------------------------------------------------------
    // CHUNK STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn chunk_length1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'0' {
            set_state!(self, ChunkLengthCr, chunk_length_cr);
        } else if is_hex!(context.byte) {
            self.length = hex_to_byte!(context.byte) as usize;

            set_state!(self, ChunkLength2, chunk_length2);
        } else {
            exit_error!(ChunkLength, context.byte);
        }

        if handler.on_chunk_begin() {
            transition_fast!(self, handler, context);
        }

        exit_callback!(self, context);
    }

    #[inline]
    fn chunk_length2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_hex!(context.byte) {
            self.length <<= 4;
            self.length  |= hex_to_byte!(context.byte) as usize;

            transition_fast!(
                self,
                handler,
                context,
                ChunkLength3,
                chunk_length3
            );
        }

        bs_replay!(context);

        transition_fast!(
            self,
            handler,
            context,
            ChunkLengthCr,
            chunk_length_cr
        );
    }

    #[inline]
    fn chunk_length3(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_hex!(context.byte) {
            self.length <<= 4;
            self.length  |= hex_to_byte!(context.byte) as usize;

            transition_fast!(
                self,
                handler,
                context,
                ChunkLength4,
                chunk_length4
            );
        }

        bs_replay!(context);

        transition_fast!(
            self,
            handler,
            context,
            ChunkLengthCr,
            chunk_length_cr
        );
    }

    #[inline]
    fn chunk_length4(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_hex!(context.byte) {
            self.length <<= 4;
            self.length  |= hex_to_byte!(context.byte) as usize;

            transition_fast!(
                self,
                handler,
                context,
                ChunkLength5,
                chunk_length5
            );
        }

        bs_replay!(context);

        transition_fast!(
            self,
            handler,
            context,
            ChunkLengthCr,
            chunk_length_cr
        );
    }

    #[inline]
    fn chunk_length5(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_hex!(context.byte) {
            self.length <<= 4;
            self.length  |= hex_to_byte!(context.byte) as usize;

            transition_fast!(
                self,
                handler,
                context,
                ChunkLength6,
                chunk_length6
            );
        }

        bs_replay!(context);

        transition_fast!(
            self,
            handler,
            context,
            ChunkLengthCr,
            chunk_length_cr
        );
    }

    #[inline]
    fn chunk_length6(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_hex!(context.byte) {
            self.length <<= 4;
            self.length  |= hex_to_byte!(context.byte) as usize;

            transition_fast!(
                self,
                handler,
                context,
                ChunkLength7,
                chunk_length7
            );
        }

        bs_replay!(context);

        transition_fast!(
            self,
            handler,
            context,
            ChunkLengthCr,
            chunk_length_cr
        );
    }

    #[inline]
    fn chunk_length7(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_hex!(context.byte) {
            self.length <<= 4;
            self.length  |= hex_to_byte!(context.byte) as usize;

            transition_fast!(
                self,
                handler,
                context,
                ChunkLength8,
                chunk_length8
            );
        }

        bs_replay!(context);

        transition_fast!(
            self,
            handler,
            context,
            ChunkLengthCr,
            chunk_length_cr
        );
    }

    #[inline]
    fn chunk_length8(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if is_hex!(context.byte) {
            self.length <<= 4;
            self.length  |= hex_to_byte!(context.byte) as usize;
        } else {
            bs_replay!(context);
        }

        transition_fast!(
            self,
            handler,
            context,
            ChunkLengthCr,
            chunk_length_cr
        );
    }

    #[inline]
    fn chunk_length_cr(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            if self.length == 0 {
                callback_transition_fast!(
                    self,
                    handler,
                    context,
                    on_chunk_length,
                    self.length,
                    HeaderLf1,
                    header_lf1
                );
            }

            callback_transition_fast!(
                self,
                handler,
                context,
                on_chunk_length,
                self.length,
                ChunkLengthLf,
                chunk_length_lf
            );
        } else if context.byte == b';' {
            callback_transition_fast!(
                self,
                handler,
                context,
                on_chunk_length,
                self.length,
                StripChunkExtensionName,
                strip_chunk_extension_name
            );
        }

        Err(ParserError::ChunkLength(context.byte))
    }

    #[inline]
    fn chunk_length_lf(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition!(
                self,
                context,
                ChunkData,
                chunk_data
            );
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn strip_chunk_extension_name(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(
            context,

            // on end-of-stream
            exit_eos!(self, context)
        );

        // make sure we have something visible to collect in next state
        if is_visible_7bit!(context.byte) {
            bs_replay!(context);

            transition_fast!(
                self,
                handler,
                context,
                LowerChunkExtensionName,
                lower_chunk_extension_name
            );
        }

        Err(ParserError::ChunkExtensionName(context.byte))
    }

    #[inline]
    fn lower_chunk_extension_name(&mut self, handler: &mut T, context: &mut ByteStream)
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
                StripChunkExtensionValue,
                strip_chunk_extension_value
            );
        } else if context.byte == b'\r' || context.byte == b';' {
            // extension name without a value
            bs_replay!(context);

            callback_transition!(
                self,
                handler,
                context,
                on_chunk_extension_name,
                ChunkExtensionFinished,
                chunk_extension_finished
            );
        } else if context.byte > 0x40 && context.byte < 0x5B {
            // upper-cased byte
            bs_replay!(context);

            callback_transition!(
                self,
                handler,
                context,
                on_chunk_extension_name,
                UpperChunkExtensionName,
                upper_chunk_extension_name
            );
        }

        Err(ParserError::ChunkExtensionName(context.byte))
    }

    #[inline]
    fn upper_chunk_extension_name(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte > 0x40 && context.byte < 0x5B {
            callback_transition_fast!(
                self,
                handler,
                context,
                on_chunk_extension_name,
                &[context.byte + 0x20],
                LowerChunkExtensionName,
                lower_chunk_extension_name
            );
        }

        bs_replay!(context);

        transition_fast!(
            self,
            handler,
            context,
            LowerChunkExtensionName,
            lower_chunk_extension_name
        );
    }

    #[inline]
    fn strip_chunk_extension_value(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(
            context,

            // on end-of-stream
            exit_eos!(self, context)
        );

        // make sure we have something visible to collect in next state
        if is_visible_7bit!(context.byte) {
            bs_replay!(context);

            transition_fast!(
                self,
                handler,
                context,
                ChunkExtensionValue,
                chunk_extension_value
            );
        }

        Err(ParserError::ChunkExtensionValue(context.byte))
    }

    #[inline]
    fn chunk_extension_value(&mut self, handler: &mut T, context: &mut ByteStream)
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

            callback_transition_fast!(
                self,
                handler,
                context,
                on_chunk_extension_value,
                ChunkExtensionFinished,
                chunk_extension_finished
            );
        } else if context.byte == b'"' {
            callback_ignore_transition_fast!(
                self,
                handler,
                context,
                on_chunk_extension_value,
                ChunkExtensionQuotedValue,
                chunk_extension_quoted_value
            );
        }

        Err(ParserError::ChunkExtensionValue(context.byte))
    }

    #[inline]
    fn chunk_extension_quoted_value(&mut self, handler: &mut T, context: &mut ByteStream)
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
                ChunkExtensionValue,
                chunk_extension_value
            );
        } else if context.byte == b'\\' {
            callback_ignore_transition!(
                self,
                handler,
                context,
                on_chunk_extension_value,
                ChunkExtensionEscapedValue,
                chunk_extension_escaped_value
            );
        }

        Err(ParserError::ChunkExtensionValue(context.byte))
    }

    #[inline]
    fn chunk_extension_escaped_value(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        // escaped bytes must be 7bit, and cannot be control characters
        if context.byte > 0x1F && context.byte < 0x7B {
            callback_transition_fast!(
                self,
                handler,
                context,
                on_chunk_extension_value,
                &[context.byte],
                ChunkExtensionQuotedValue,
                chunk_extension_quoted_value
            );
        }

        Err(ParserError::ChunkExtensionValue(context.byte))
    }

    #[inline]
    fn chunk_extension_finished(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            set_state!(self, ChunkExtensionsFinished, chunk_extensions_finished);
        } else {
            set_state!(self, StripChunkExtensionName, strip_chunk_extension_name);
        }

        if handler.on_chunk_extension_finished() {
            transition!(self, context);
        }

        exit_callback!(self, context);
    }

    #[inline]
    fn chunk_extensions_finished(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        set_state!(self, ChunkLengthLf, chunk_length_lf);

        if handler.on_chunk_extensions_finished() {
            transition!(
                self,
                context
            );
        }

        exit_callback!(self, context);
    }

    #[inline]
    fn chunk_data(&mut self, handler: &mut T, context: &mut ByteStream)
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
                ChunkDataCr,
                chunk_data_cr
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
            ChunkData,
            chunk_data
        );
    }

    #[inline]
    fn chunk_data_cr(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition_fast!(
                self,
                handler,
                context,
                ChunkDataLf,
                chunk_data_lf
            );
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    #[inline]
    fn chunk_data_lf(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition_fast!(
                self,
                handler,
                context,
                ChunkLength1,
                chunk_length1
            );
        }

        Err(ParserError::CrlfSequence(context.byte))
    }

    // ---------------------------------------------------------------------------------------------
    // MULTIPART STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn multipart_hyphen1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'-' {
            transition_fast!(
                self,
                handler,
                context,
                MultipartHyphen2,
                multipart_hyphen2
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
                MultipartDataByByte,
                multipart_data_by_byte
            );
        }

        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    fn multipart_hyphen2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'-' {
            transition_fast!(
                self,
                handler,
                context,
                MultipartBoundary,
                multipart_boundary
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
                MultipartDataByByte,
                multipart_data_by_byte
            );
        }

        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    fn multipart_boundary(&mut self, handler: &mut T, context: &mut ByteStream)
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
                    MultipartDataByByte,
                    multipart_data_by_byte
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
                MultipartBoundaryCr,
                multipart_boundary_cr
            );
        }

        // boundary comparison not finished
        inc_upper14!(self, length);

        exit_eos!(self, context);
    }

    #[inline]
    fn multipart_boundary_cr(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            set_state!(self, InitialLf, initial_lf);

            if handler.on_multipart_begin() {
                transition!(self, context);
            }

            exit_callback!(self, context);
        } else if context.byte == b'-' {
            transition_fast!(
                self,
                handler,
                context,
                MultipartEnd,
                multipart_end
            );
        }

        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    fn multipart_boundary_lf(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition!(
                self,
                context,
                CheckHeaderName,
                check_header_name
            );
        }

        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    fn multipart_detect_data(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        if let Some(length) = handler.content_length() {
            self.length = length;

            // expect boundary after data
            set_lower14!(self, 1);

            transition_fast!(
                self,
                handler,
                context,
                MultipartDataByLength,
                multipart_data_by_length
            );
        }

        // do not expect boundary since it can be part of the data itself
        set_lower14!(self, 0);

        transition_fast!(
            self,
            handler,
            context,
            MultipartDataByByte,
            multipart_data_by_byte
        );
    }

    #[inline]
    fn multipart_data_by_length(&mut self, handler: &mut T, context: &mut ByteStream)
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
                MultipartDataByLengthCr,
                multipart_data_by_length_cr
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
            MultipartDataByLength,
            multipart_data_by_length
        );
    }

    #[inline]
    fn multipart_data_by_length_cr(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\r' {
            transition_fast!(
                self,
                handler,
                context,
                MultipartDataByLengthLf,
                multipart_data_by_length_lf
            );
        }

        // this state is only used after multipart_data_by_length, so we can error if we don't
        // find the carriage return
        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    fn multipart_data_by_length_lf(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition_fast!(
                self,
                handler,
                context,
                MultipartHyphen1,
                multipart_hyphen1
            );
        }

        // this state is only used after multipart_data_by_length, so we can error if we don't
        // find the carriage return
        Err(ParserError::MultipartBoundary(context.byte))
    }

    #[inline]
    fn multipart_data_by_byte(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        bs_collect_until!(
            context,

            // collect bytes until
            context.byte == b'\r',

            // on end-of-stream
            callback_eos_expr!(self, handler, context, on_multipart_data)
        );

        callback_ignore_transition_fast!(
            self,
            handler,
            context,
            on_multipart_data,
            MultipartDataByByteLf,
            multipart_data_by_byte_lf
        );
    }

    #[inline]
    fn multipart_data_by_byte_lf(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'\n' {
            transition_fast!(
                self,
                handler,
                context,
                MultipartHyphen1,
                multipart_hyphen1
            );
        }

        callback_transition!(
            self,
            handler,
            context,
            on_multipart_data,
            &[b'\r', context.byte],
            MultipartDataByByte,
            multipart_data_by_byte
        );
    }

    #[inline]
    fn multipart_end(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);
        bs_next!(context);

        if context.byte == b'-' {
            transition!(
                self,
                context,
                BodyFinished,
                body_finished
            );
        }

        Err(ParserError::MultipartBoundary(context.byte))
    }

    // ---------------------------------------------------------------------------------------------
    // URL ENCODED STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn url_encoded_name(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        collect_visible_7bit!(
            context,

            // stop on these bytes
            is_url_encoded_separator(context.byte),

            // on end-of-stream
            callback_eos_expr!(self, handler, context, on_url_encoded_name)
        );

        match context.byte {
            b'=' => {
                callback_ignore_transition_fast!(
                    self,
                    handler,
                    context,
                    on_url_encoded_name,
                    UrlEncodedValue,
                    url_encoded_value
                );
            },
            b'%' => {
                callback_ignore_transition_fast!(
                    self,
                    handler,
                    context,
                    on_url_encoded_name,
                    UrlEncodedNameHex1,
                    url_encoded_name_hex1
                );
            },
            b'&' | b';' => {
                callback_ignore_transition_fast!(
                    self,
                    handler,
                    context,
                    on_url_encoded_name,
                    UrlEncodedNameAmpersand,
                    url_encoded_name_ampersand
                );
            },
            b'+' => {
                callback_ignore_transition_fast!(
                    self,
                    handler,
                    context,
                    on_url_encoded_name,
                    UrlEncodedNamePlus,
                    url_encoded_name_plus
                );
            },
            _ => {
                Err(ParserError::UrlEncodedName(context.byte))
            }
        }
    }

    #[inline]
    fn url_encoded_name_ampersand(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        // no value, send an empty one
        callback_transition!(
            self,
            handler,
            context,
            on_url_encoded_value,
            b"",
            UrlEncodedName,
            url_encoded_name
        );
    }

    #[inline]
    fn url_encoded_name_hex1(&mut self, handler: &mut T, context: &mut ByteStream)
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

        transition_fast!(
            self,
            handler,
            context,
            UrlEncodedNameHex2,
            url_encoded_name_hex2
        );
    }

    #[inline]
    fn url_encoded_name_hex2(&mut self, handler: &mut T, context: &mut ByteStream)
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
            UrlEncodedName,
            url_encoded_name
        );
    }

    #[inline]
    fn url_encoded_name_plus(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        callback_transition!(
            self,
            handler,
            context,
            on_url_encoded_name,
            b" ",
            UrlEncodedName,
            url_encoded_name
        );
    }

    #[inline]
    fn url_encoded_value(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        collect_visible_7bit!(
            context,

            // stop on these bytes
            is_url_encoded_separator(context.byte),

            // on end-of-stream
            callback_eos_expr!(self, handler, context, on_url_encoded_value)
        );

        match context.byte {
            b'%' => {
                callback_ignore_transition_fast!(
                    self,
                    handler,
                    context,
                    on_url_encoded_value,
                    UrlEncodedValueHex1,
                    url_encoded_value_hex1
                );
            },
            b'&' | b';' => {
                callback_ignore_transition!(
                    self,
                    handler,
                    context,
                    on_url_encoded_value,
                    UrlEncodedName,
                    url_encoded_name
                );
            },
            b'+' => {
                callback_ignore_transition_fast!(
                    self,
                    handler,
                    context,
                    on_url_encoded_value,
                    UrlEncodedValuePlus,
                    url_encoded_value_plus
                );
            },
              b'='
            | _ => {
                Err(ParserError::UrlEncodedValue(context.byte))
            }
        }
    }

    #[inline]
    fn url_encoded_value_hex1(&mut self, handler: &mut T, context: &mut ByteStream)
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

        transition_fast!(
            self,
            handler,
            context,
            UrlEncodedValueHex2,
            url_encoded_value_hex2
        );
    }

    #[inline]
    fn url_encoded_value_hex2(&mut self, handler: &mut T, context: &mut ByteStream)
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
            UrlEncodedValue,
            url_encoded_value
        );
    }

    #[inline]
    fn url_encoded_value_plus(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        callback_transition!(
            self,
            handler,
            context,
            on_url_encoded_value,
            b" ",
            UrlEncodedValue,
            url_encoded_value
        );
    }

    // ---------------------------------------------------------------------------------------------
    // DEAD & FINISHED STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn dead(&mut self, _handler: &mut T, _context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_error!(Dead);
    }

    #[inline]
    fn body_finished(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        set_state!(self, Finished, finished);

        if handler.on_body_finished() {
            transition_fast!(self, handler, context);
        }

        exit_callback!(self, context);
    }

    #[inline]
    fn finished(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_finished!(self, context);
    }
}
