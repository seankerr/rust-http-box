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

use byte::hex_to_byte;
use byte::is_token;

/// Maximum chunk extension byte count to process before returning
/// `ParserError::MaxChunkExtensionLength`.
pub const CFG_MAX_CHUNK_EXTENSION_LENGTH: u8 = 255;

/// Maximum chunk size byte count to process before returning `ParserError::MaxChunkSizeLength`.
pub const CFG_MAX_CHUNK_SIZE_LENGTH: u8 = 16;

/// Maximum headers byte count to process before returning `ParserError::MaxHeadersLength`.
pub const CFG_MAX_HEADERS_LENGTH: u32 = 80 * 1024;

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

/// Maximum chunk size length has been met.
pub const ERR_MAX_CHUNK_SIZE_LENGTH: &'static str = "Maximum chunk size length";

/// Maximum headers length has been met.
pub const ERR_MAX_HEADERS_LENGTH: &'static str = "Maximum headers length";

/// Invalid method.
pub const ERR_METHOD: &'static str = "Invalid method";

/// Missing content length header.
pub const ERR_MISSING_CONTENT_LENGTH: &'static str = "Missing Content-Length header";

/// Invalid status.
pub const ERR_STATUS: &'static str = "Invalid status";

/// Invalid status code.
pub const ERR_STATUS_CODE: &'static str = "Invalid status code";

/// Invalid URL.
pub const ERR_URL: &'static str = "Invalid URL";

/// Invalid version.
pub const ERR_VERSION: &'static str = "Invalid HTTP version";

// -------------------------------------------------------------------------------------------------

#[allow(dead_code)]
enum Callback<T> {
    None,
    Data(fn(&mut T, &[u8]) -> bool),
    Empty(fn(&mut T,) -> bool),
}

/// Connection.
#[derive(Clone,Copy,PartialEq)]
#[repr(u8)]
pub enum Connection {
    None,
    Close,
    Upgrade
}

/// Content length.
#[derive(Clone,Copy,PartialEq)]
pub enum ContentLength {
    None,
    Specified(u64)
}

/// Content type.
pub enum ContentType<'a> {
    None,
    Multipart(&'a [u8]),
    UrlEncoded(&'a mut Vec<u8>),
    Other(&'a [u8]),
}

/// Parser error messages.
#[derive(Clone,Copy,PartialEq)]
pub enum ParserError {
    /// Invalid chunk extension.
    ChunkExtension(&'static str, u8),

    /// Invalid chunk size.
    ChunkSize(&'static str, u8),

    /// Returned when an invalid CRLF sequence is found.
    CrlfSequence(&'static str),

    /// Returned when parsing has failed, but `Parser::parse()` is executed again.
    Dead(&'static str),

    /// Returned when the parser expects more data.
    Eof,

    /// Returned when a header field is invalid.
    HeaderField(&'static str, u8),

    /// Returned when a header value is invalid.
    HeaderValue(&'static str, u8),

    /// Returned when maximum chunk extension length has been met.
    MaxChunkExtensionLength(&'static str, u8),

    /// Returned when maximum chunk size has been met.
    MaxChunkSizeLength(&'static str, u8),

    /// Returned when maximum headers length has been met.
    MaxHeadersLength(&'static str, u32),

    /// Missing an expected Content-Length header.
    MissingContentLength(&'static str),

    /// Returned when the method is invalid.
    Method(&'static str, u8),

    /// Returned when the status is invalid.
    Status(&'static str, u8),

    /// Returned when the status code is invalid.
    StatusCode(&'static str),

    /// Returned when a URL has an invalid character.
    Url(&'static str, u8),

    /// Returned when the HTTP major/minor version is invalid.
    Version(&'static str),
}

/// Indicates the current parser state.
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

    /// Parsing request method.
    RequestMethod,

    /// Determining if URL starts with a scheme, or is an absolute path
    RequestUrl,

    /// Parsing request HTTP version.
    RequestHttp1,
    RequestHttp2,
    RequestHttp3,
    RequestHttp4,
    RequestHttp5,

    /// Parsing request HTTP major version.
    RequestVersionMajor,

    /// Parsing request HTTP minor version.
    RequestVersionMinor,

    // ---------------------------------------------------------------------------------------------
    // RESPONSE
    // ---------------------------------------------------------------------------------------------

    /// Parsing response HTTP version.
    ResponseHttp1,
    ResponseHttp2,
    ResponseHttp3,
    ResponseHttp4,
    ResponseHttp5,

    /// Parsing response HTTP major version.
    ResponseVersionMajor,

    /// Parsing response HTTP minor version.
    ResponseVersionMinor,

    /// Parsing response status code.
    ResponseStatusCode,

    /// Parsing response status.
    ResponseStatus,

    // ---------------------------------------------------------------------------------------------
    // HEADERS
    // ---------------------------------------------------------------------------------------------

    /// Parsing pre-header CRLF[CR].
    // note: these only exist purely to avoid the situation where a client can send an initial
    //       request/response line, then CRLF[SPACE], and the parser would have assumed the next
    //       piece of content is the second line of a multiline header value
    //
    //       in addition to this, multiline header value support has been deprecated, but we'll keep
    //       support for now: https://tools.ietf.org/html/rfc7230#section-3.2.4
    PreHeaders1,
    PreHeaders2,
    PreHeaders3,

    /// Parsing header field.
    HeaderField,

    /// Stripping space before header value.
    StripHeaderValue,

    /// Parsing header value.
    HeaderValue,

    /// Parsing header quoted value.
    QuotedHeaderValue,

    /// CRLF[CRLF] after header value
    Newline1,
    Newline2,
    Newline3,
    Newline4,

    // ---------------------------------------------------------------------------------------------
    // BODY
    // ---------------------------------------------------------------------------------------------

    /// Parsing body.
    Body,

    /// Binary or text body content.
    BodyContent,

    /// Parsing chunk size.
    ChunkSize,

    /// Parsing chunk extension.
    ChunkExtension,

    /// CRLF after chunk size.
    ChunkNewline,

    /// Parsing chunk data.
    ChunkData,

    /// Parsing multipart starting boundary.
    MultipartStartingBoundary,

    /// Parsing multipart data.
    MultipartData,

    /// Parsing multipart ending boundary.
    MultipartEndingBoundary,

    /// Parsing multipart finished.
    MultipartFinished,

    /// Parsing URL encoded data.
    UrlEncoded,

    // ---------------------------------------------------------------------------------------------

    /// Parser finished successfully.
    Finished
}

/// Stream type.
#[derive(Clone,Copy,PartialEq)]
#[repr(u8)]
pub enum StreamType {
    /// Request stream type.
    Request,

    /// Response stream type.
    Response
}

/// Transfer encoding.
#[derive(Clone,Copy,PartialEq)]
#[repr(u8)]
pub enum TransferEncoding {
    None,
    Chunked
}

// -------------------------------------------------------------------------------------------------

/// Type that handles HTTP parser events.
#[allow(unused_variables)]
pub trait HttpHandler {
    /// Retrieve the most recent Connection header value.
    fn get_connection(&mut self) -> Connection {
        Connection::None
    }

    /// Retrieve the most recent Content-Length header value in numeric representation.
    fn get_content_length(&mut self) -> ContentLength {
        ContentLength::None
    }

    /// Retrieve the most recent Content-Type header value.
    fn get_content_type(&mut self) -> ContentType {
        ContentType::None
    }

    /// Retrieve the most recent Transfer-Encoding header value.
    fn get_transfer_encoding(&mut self) -> TransferEncoding {
        TransferEncoding::None
    }

    /// Callback that is executed when raw body data has been received.
    ///
    /// This may be executed multiple times in order to supply the entire body.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_body(&mut self, data: &[u8]) -> bool {
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
    fn on_chunk_extension(&mut self, extensions: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a chunk size has been located.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_chunk_size(&mut self, size: u64) -> bool {
        true
    }

    /// Callback that is executed when parsing has completed successfully.
    fn on_finished(&mut self) {
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
    fn on_status_code(&mut self, status_code: u16) -> bool {
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

    /// Callback that is executed when the HTTP version has been located.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_version(&mut self, major: u16, minor: u16) -> bool {
        true
    }
}

// -------------------------------------------------------------------------------------------------

// Flags used to track individual byte details.
bitflags! {
    flags ByteFlag: u8 {
        // No flags.
        const B_NONE = 0,

        // Forget most previous byte.
        const B_FORGET = 1 << 0,

        // Replay the current byte.
        const B_REPLAY = 1 << 1
    }
}

// Flags used to track request/response/state details.
bitflags! {
    flags Flag: u8 {
        // No flags.
        const F_NONE = 0,

        // Parsing initial.
        const F_IN_INITIAL = 1 << 0,

        // Parsing headers.
        const F_IN_HEADERS = 1 << 1,

        // Finished parsing headers.
        const F_HEADERS_FINISHED = 1 << 2,

        // Quoted header value has an escape character.
        const F_QUOTE_ESCAPED = 1 << 3,

        // Indicates the stream contains request data rather than response.
        const F_REQUEST = 1 << 4
    }
}

pub struct Parser<'a, T: HttpHandler> {
    // Total bytes processed since the start of the request/response message.
    // This is updated each time Parser::parse() returns.
    // This resets on each new request/response message.
    byte_count: usize,

    // Current lazy callback that is handled at the top of each byte loop.
    callback: Callback<T>,

    // Content type.
    content_type: ContentType<'a>,

    // The request/response flags.
    flags: Flag,

    // Maximum header byte count to process before we assume it's a DoS stream.
    max_headers_length: u32,

    // Index used for overflow detection.
    overflow_index: u8,

    // Count of remaining bytes to read.
    // This is used when reading chunks, or when reading content with a set length.
    remaining_byte_count: u64,

    // Current state.
    state: State,

    // Response status code.
    status_code: u16,

    // HTTP major version.
    version_major: u16,

    // HTTP minor version.
    version_minor: u16
}

// -------------------------------------------------------------------------------------------------

impl<'a, T: HttpHandler> Parser<'a, T> {
    /// Create a new `Parser` with a maximum headers length of `80 * 1024`.
    pub fn new(stream_type: StreamType) -> Parser<'a, T> {
        Parser{ byte_count:           0,
                callback:             Callback::None,
                content_type:         ContentType::None,
                flags:                if stream_type == StreamType::Request {
                                          F_IN_INITIAL | F_REQUEST
                                      } else {
                                          F_IN_INITIAL
                                      },
                max_headers_length:   CFG_MAX_HEADERS_LENGTH,
                overflow_index:       0,
                remaining_byte_count: 0,
                state:                if stream_type == StreamType::Request {
                                          State::RequestMethod
                                      } else {
                                          State::ResponseHttp1
                                      },
                status_code:          0,
                version_major:        0,
                version_minor:        0 }
    }

    /// Create a new `Parser` with specified settings.
    pub fn with_settings(stream_type: StreamType, max_headers_length: u32) -> Parser<'a, T> {
        Parser{ byte_count:           0,
                callback:             Callback::None,
                content_type:         ContentType::None,
                flags:                if stream_type == StreamType::Request {
                                          F_IN_INITIAL | F_REQUEST
                                      } else {
                                          F_IN_INITIAL
                                      },
                max_headers_length:   max_headers_length,
                overflow_index:       0,
                remaining_byte_count: 0,
                state:                if stream_type == StreamType::Request {
                                          State::RequestMethod
                                      } else {
                                          State::ResponseHttp1
                                      },
                status_code:          0,
                version_major:        0,
                version_minor:        0 }
    }

    /// Retrieve the processed byte count since the start of the message.
    pub fn get_byte_count(&self) -> usize {
        self.byte_count
    }

    /// Retrieve the current parser state.
    pub fn get_state(&self) -> State {
        self.state
    }

    /// Parse HTTP data.
    ///
    /// If `Ok()` is returned, one of two things have happened: parsing has finished, or
    /// a callback function has preemptively stopped parsing. In either of these events, it
    /// is ok to call `parse()` again with a new slice of data to continue where the parser
    /// left off. `ParserError::Eof` is the only `ParserError` that allows `parse()` to
    /// continue parsing. Any other `ParserError` is a protocol error.
    ///
    /// Returns the parsed byte count of the current slice when parsing completes, or when
    /// a callback function preemptively stops parsing. Otherwise `ParserError`.
    #[cfg_attr(test, allow(cyclomatic_complexity))]
    pub fn parse(&mut self, handler: &mut T, stream: &[u8]) -> Result<usize, ParserError> {
        // current byte
        let mut byte: u8 = 0;

        // byte flags
        let mut byte_flags: ByteFlag = B_NONE;

        // lazy callback to execute
        let mut callback = match self.callback {
            Callback::Data(x)  => Callback::Data(x),
            Callback::Empty(x) => Callback::Empty(x),
            Callback::None     => Callback::None
        };

        // message flags
        let mut flags = self.flags;

        // stream index for the start of the mark
        let mut mark_index: usize = 0;

        // stream index at which to check max headers length
        let max_headers_length_index: usize = if self.max_headers_length > 0 {
                                                    self.max_headers_length
                                                  - self.byte_count as u32
                                                  + 1
                                              } else {
                                                  0
                                              } as usize;

        // old state
        let mut old_state = self.state;

        // overflow index
        let mut overflow_index = self.overflow_index;

        // remaining byte count
        let mut remaining_byte_count = self.remaining_byte_count;

        // current state
        let mut state = self.state;

        // stream index we're processing
        let mut stream_index: usize = 0;

        if state == State::Dead {
            return Err(ParserError::Dead(ERR_DEAD))
        }

        // -----------------------------------------------------------------------------------------

        // check max headers length
        macro_rules! check_max_headers_length {
            () => (
                if stream_index == max_headers_length_index
                && flags.bits & F_HEADERS_FINISHED.bits == F_NONE.bits {
                    error!(ParserError::MaxHeadersLength(ERR_MAX_HEADERS_LENGTH,
                                                         self.max_headers_length));
                }
            );
        }

        // collect macro base
        macro_rules! collect_base {
            ($block:block) => ({
                let mut found = false;

                // put stream index back one byte to reflect our start loop index
                stream_index -= 1;

                while !is_eof!() {
                    byte = peek!();

                    if $block {
                        found         = true;
                        stream_index += 1;

                        break
                    }

                    stream_index += 1;

                    check_max_headers_length!();
                }

                found
            });
        }

        // collect a digit
        macro_rules! collect_digit {
            ($byte:expr, $digit:expr, $max:expr, $error:path, $error_msg:expr) => (
                collect_base!({
                    if is_digit!(byte) {
                        $digit *= 10;
                        $digit += byte as u16 - b'0' as u16;

                        if $digit > $max {
                            error!($error($error_msg));
                        }

                        false
                    } else if $byte == byte {
                        true
                    } else {
                        error!($error($error_msg));
                    }
                })
            );
        }

        // collect a hex digit
        macro_rules! collect_hex_digit {
            ($byte1:expr, $byte2:expr, $max:expr, $overflow_error:path, $overflow_error_msg:expr,
             $error:path, $error_msg:expr) => (
                collect_base!({
                    if is_hex!(byte) {
                        if $max == overflow_index {
                            error!($overflow_error($overflow_error_msg, byte));
                        }

                        overflow_index        += 1;
                        remaining_byte_count <<= 4;

                        match hex_to_byte(&[byte]) {
                            Some(byte) => {
                                remaining_byte_count += byte as u64;
                            },
                            None => {
                                error!($error($error_msg, byte));
                            }
                        }

                        false
                    } else if $byte1 == byte || $byte2 == byte {
                        true
                    } else {
                        error!($error($error_msg, byte));
                    }
                })
            );
        }

        // collect non-control characters
        macro_rules! collect_non_control {
            () => (
                collect_base!({
                    is_control!(byte)
                })
            );
        }

        // collect only the given characters
        macro_rules! collect_only {
            ($byte:expr) => (
                collect_base!({
                    if $byte == byte {
                        false
                    } else {
                        true
                    }
                })
            );

            ($byte1:expr, $byte2:expr) => (
                collect_base!({
                    !($byte1 == byte || $byte2 == byte)
                })
            );
        }

        // collect non-control characters until a certain byte is found
        macro_rules! collect_until {
            ($byte1:expr, $byte2:expr, $error:path, $error_msg:expr) => (
                collect_base!({
                    if $byte1 == byte || $byte2 == byte {
                        true
                    } else if !is_ascii!(byte) || is_control!(byte) {
                        error!($error($error_msg, byte));
                    } else {
                        false
                    }
                })
            );

            ($byte:expr, $error:path, $error_msg:expr) => (
                collect_base!({
                    if $byte == byte {
                        true
                    } else if !is_ascii!(byte) || is_control!(byte) {
                        error!($error($error_msg, byte));
                    } else {
                        false
                    }
                })
            );
        }

        // collect token characters until a certain byte is found
        macro_rules! collect_token_until {
            ($byte:expr, $error:path, $error_msg:expr) => (
                collect_base!({
                    if $byte == byte {
                        true
                    } else if is_token(byte) {
                        false
                    } else {
                        error!($error($error_msg, byte));
                    }
                })
            );
        }

        // collect token characters until a certain byte is found, but check an overflow
        macro_rules! collect_token_until_overflow {
            ($byte:expr, $max:expr, $overflow_error:path, $overflow_error_msg:expr,
             $error:path, $error_msg:expr) => (
                collect_base!({
                    if $byte == byte {
                        true
                    } else if is_token(byte) {
                        if $max == overflow_index {
                            error!($overflow_error($overflow_error_msg, byte));
                        }

                        overflow_index += 1;

                        false
                    } else {
                        error!($error($error_msg, byte));
                    }
                })
            );
        }

        // collect token characters, spaces, and tabs until a certain byte is found
        macro_rules! collect_token_space_tab_until {
            ($byte:expr, $error:path, $error_msg:expr) => (
                collect_base!({
                    if $byte == byte {
                        true
                    } else if is_token(byte) || byte == b' ' || byte == b'\t' {
                        false
                    } else {
                        error!($error($error_msg, byte));
                    }
                })
            );
        }

        // set the state to State::Dead and return an error
        macro_rules! error {
            ($error:expr) => (
                self.state = State::Dead;

                return Err($error);
            );
        }

        // save state and exit
        macro_rules! exit_eof {
            () => (
                self.byte_count           += stream_index;
                self.callback              = callback;
                self.flags                 = flags;
                self.overflow_index        = overflow_index;
                self.remaining_byte_count  = remaining_byte_count;
                self.state                 = state;

                return Err(ParserError::Eof);
            );
        }

        macro_rules! exit_ok {
            () => (
                self.byte_count           += stream_index;
                self.callback              = callback;
                self.flags                 = flags;
                self.overflow_index        = overflow_index;
                self.remaining_byte_count  = remaining_byte_count;
                self.state                 = state;

                return Ok(stream_index);
            );

            ($state:expr) => (
                self.byte_count           += stream_index;
                self.callback              = callback;
                self.flags                 = flags;
                self.overflow_index        = overflow_index;
                self.remaining_byte_count  = remaining_byte_count;
                self.state                 = $state;

                return Ok(stream_index);
            );
        }

        // forget one byte
        macro_rules! forget {
            () => (
                byte_flags.insert(B_FORGET);
            );
        }

        // indicates that we have enough bytes in the stream to extract them
        macro_rules! has_bytes {
            ($count:expr) => (
                stream_index + $count - 1 < stream.len()
            );
        }

        // check end of stream
        macro_rules! is_eof {
            () => (
                stream_index == stream.len()
            );
        }

        // jump a specific number of bytes
        macro_rules! jump {
            ($count:expr) => (
                stream_index += $count;
                byte          = stream[stream_index-1];

                // check max headers length
                // we're incrementing the stream index by an arbitrary amount of bytes, so we cannot
                // check max_headers_length_index == stream_index
                if flags.bits & F_HEADERS_FINISHED.bits == F_NONE.bits
                && stream_index > max_headers_length_index {
                    error!(ParserError::MaxHeadersLength(ERR_MAX_HEADERS_LENGTH,
                                                         self.max_headers_length));
                }
            );
        }

        // save a callback to be executed lazily by the parser
        macro_rules! lazy_callback {
            // callback without data
            ($function:ident) => (
                callback = Callback::Empty(T::$function);
            );

            // callback with data
            ($function:ident, $_has_data:expr) => (
                callback = Callback::Data(T::$function);
            );
        }

        // mark the current byte as the first mark byte
        macro_rules! mark {
            () => (
                mark_index = stream_index - 1;
            );
        }

        // get the marked bytes
        macro_rules! marked_bytes {
            () => (
                &stream[mark_index..stream_index - (byte_flags.bits & B_FORGET.bits) as usize]
            );
        }

        // skip to the next byte
        macro_rules! next {
            () => (
                if is_eof!() {
                    exit_eof!();
                }

                byte          = peek!();
                byte_flags    = B_NONE;
                stream_index += 1;

                // check max headers length
                if stream_index == max_headers_length_index
                && flags.bits & F_HEADERS_FINISHED.bits == F_NONE.bits {
                    error!(ParserError::MaxHeadersLength(ERR_MAX_HEADERS_LENGTH,
                                                         self.max_headers_length));
                }
            );
        }

        // peek at the next byte
        macro_rules! peek {
            () => (
                stream[stream_index]
            );

            ($count:expr) => (
                stream[stream_index + $count - 1]
            )
        }

        // peek at a chunk of bytes starting with the current byte
        macro_rules! peek_chunk {
            ($count:expr) => (
                &stream[stream_index - 1..stream_index + $count - 1]
            );
        }

        // replay the current byte
        macro_rules! replay {
            () => (
                byte_flags.insert(B_REPLAY);
            );
        }

        // skip to a new state and bypass the match loop
        macro_rules! skip_to_state {
            ($state:expr) => (
                state = $state;

                top_of_loop!();
            );
        }

        // top of loop processing
        macro_rules! top_of_loop {
            () => (
                if state != old_state {
                    match callback {
                        Callback::Data(x) => {
                            callback = Callback::None;

                            if !x(handler, marked_bytes!()) {
                                exit_ok!();
                            }
                        },
                        Callback::Empty(x) => {
                            callback = Callback::None;

                            if !x(handler) {
                                exit_ok!();
                            }
                        },
                        Callback::None => {
                        }
                    }
                } else if is_eof!() {
                    match callback {
                        Callback::Data(x) => {
                            if !x(handler, marked_bytes!()) {
                                exit_ok!();
                            }
                        },
                        Callback::Empty(x) => {
                            if !x(handler) {
                                exit_ok!();
                            }
                        },
                        Callback::None => {
                        }
                    }
                }

                if byte_flags.contains(B_REPLAY) {
                    byte_flags = B_NONE;
                } else {
                    next!();
                }

                if state != old_state {
                    mark!();
                }

                old_state = state;
            );
        }

        // -----------------------------------------------------------------------------------------
        // STATE MACROS IN ORDER OF EXECUTION
        // -----------------------------------------------------------------------------------------

        // private request method macros
        macro_rules! request_method {
            () => ({
                lazy_callback!(on_method, true);

                if collect_token_until!(b' ', ParserError::Method, ERR_METHOD) {
                    forget!();

                    skip_to_state!(State::RequestUrl);
                    state_RequestUrl!()
                } else {
                    State::RequestMethod
                }
            });
        }

        macro_rules! request_method_handler {
            ($method:expr) => (
                if handler.on_method($method) {
                    skip_to_state!(State::RequestUrl);
                    state_RequestUrl!()
                } else {
                    exit_ok!(State::RequestUrl);
                }
            );
        }

        macro_rules! state_RequestMethod {
            () => (
                if has_bytes!(7) {
                    if b"GET " == peek_chunk!(4) {
                        jump!(3);
                        request_method_handler!(b"GET")
                    } else if b"POST " == peek_chunk!(5) {
                        jump!(4);
                        request_method_handler!(b"POST")
                    } else if b"PUT " == peek_chunk!(4) {
                        jump!(3);
                        request_method_handler!(b"PUT")
                    } else if b"DELETE " == peek_chunk!(7) {
                        jump!(6);
                        request_method_handler!(b"DELETE")
                    } else if b"CONNECT " == peek_chunk!(8) {
                        jump!(7);
                        request_method_handler!(b"CONNECT")
                    } else if b"OPTIONS " == peek_chunk!(8) {
                        jump!(7);
                        request_method_handler!(b"OPTIONS")
                    } else if b"HEAD " == peek_chunk!(5) {
                        jump!(4);
                        request_method_handler!(b"HEAD")
                    } else if b"TRACE " == peek_chunk!(6) {
                        jump!(5);
                        request_method_handler!(b"TRACE")
                    } else {
                        request_method!()
                    }
                } else {
                    request_method!()
                }
            );
        }

        macro_rules! state_RequestUrl {
            () => ({
                lazy_callback!(on_url, true);

                if collect_until!(b' ', ParserError::Url, ERR_URL) {
                    forget!();
                    skip_to_state!(State::RequestHttp1);
                    state_RequestHttp1!()
                } else {
                    State::RequestUrl
                }
            });
        };

        macro_rules! state_RequestHttp1 {
            () => (
                if has_bytes!(4) && (b"HTTP/" == peek_chunk!(5) || b"http/" == peek_chunk!(5)) {
                    jump!(4);
                    skip_to_state!(State::RequestVersionMajor);
                    state_RequestVersionMajor!()
                } else if byte == b'H' || byte == b'h' {
                    skip_to_state!(State::RequestHttp2);
                    state_RequestHttp2!()
                } else {
                    error!(ParserError::Version(ERR_VERSION));
                }
            );
        };

        macro_rules! state_RequestHttp2 {
            () => (
                if byte == b'T' || byte == b't' {
                    skip_to_state!(State::RequestHttp3);
                    state_RequestHttp3!()
                } else {
                    error!(ParserError::Version(ERR_VERSION));
                }
            );
        };

        macro_rules! state_RequestHttp3 {
            () => (
                if byte == b'T' || byte == b't' {
                    skip_to_state!(State::RequestHttp4);
                    state_RequestHttp4!()
                } else {
                    error!(ParserError::Version(ERR_VERSION));
                }
            );
        };

        macro_rules! state_RequestHttp4 {
            () => (
                if byte == b'P' || byte == b'p' {
                    skip_to_state!(State::RequestHttp5);
                    state_RequestHttp5!()
                } else {
                    error!(ParserError::Version(ERR_VERSION));
                }
            );
        };

        macro_rules! state_RequestHttp5 {
            () => (
                if byte == b'/' {
                    skip_to_state!(State::RequestVersionMajor);
                    state_RequestVersionMajor!()
                } else {
                    error!(ParserError::Version(ERR_VERSION));
                }
            );
        };

        macro_rules! state_RequestVersionMajor {
            () => ({
                if collect_digit!(b'.', self.version_major, 999,
                                  ParserError::Version, ERR_VERSION) {
                    skip_to_state!(State::RequestVersionMinor);
                    state_RequestVersionMinor!()
                } else {
                    State::RequestVersionMajor
                }
            });
        }

        macro_rules! state_RequestVersionMinor {
            () => ({
                if collect_digit!(b'\r', self.version_minor, 999,
                                  ParserError::Version, ERR_VERSION) {
                    if handler.on_version(self.version_major, self.version_minor) {
                        skip_to_state!(State::PreHeaders1);
                        state_PreHeaders1!()
                    } else {
                        exit_ok!(State::PreHeaders1);
                    }
                } else {
                    State::RequestVersionMinor
                }
            });
        }

        macro_rules! state_ResponseHttp1 {
            () => (
                if has_bytes!(4) && (b"HTTP/" == peek_chunk!(5) || b"http/" == peek_chunk!(5)) {
                    jump!(4);
                    skip_to_state!(State::ResponseVersionMajor);
                    state_ResponseVersionMajor!()
                } else if byte == b'H' || byte == b'h' {
                    skip_to_state!(State::ResponseHttp2);
                    state_ResponseHttp2!()
                } else {
                    error!(ParserError::Version(ERR_VERSION));
                }
            );
        };

        macro_rules! state_ResponseHttp2 {
            () => (
                if byte == b'T' || byte == b't' {
                    skip_to_state!(State::ResponseHttp3);
                    state_ResponseHttp3!()
                } else {
                    error!(ParserError::Version(ERR_VERSION));
                }
            );
        };

        macro_rules! state_ResponseHttp3 {
            () => (
                if byte == b'T' || byte == b't' {
                    skip_to_state!(State::ResponseHttp4);
                    state_ResponseHttp4!()
                } else {
                    error!(ParserError::Version(ERR_VERSION));
                }
            );
        };

        macro_rules! state_ResponseHttp4 {
            () => (
                if byte == b'P' || byte == b'p' {
                    skip_to_state!(State::ResponseHttp5);
                    state_ResponseHttp5!()
                } else {
                    error!(ParserError::Version(ERR_VERSION));
                }
            );
        };

        macro_rules! state_ResponseHttp5 {
            () => (
                if byte == b'/' {
                    skip_to_state!(State::ResponseVersionMajor);
                    state_ResponseVersionMajor!()
                } else {
                    error!(ParserError::Version(ERR_VERSION));
                }
            );
        };

        macro_rules! state_ResponseVersionMajor {
            () => ({
                if collect_digit!(b'.', self.version_major, 999,
                                  ParserError::Version, ERR_VERSION) {
                    skip_to_state!(State::ResponseVersionMinor);
                    state_ResponseVersionMinor!()
                } else {
                    State::ResponseVersionMajor
                }
            });
        }

        macro_rules! state_ResponseVersionMinor {
            () => ({
                if collect_digit!(b' ', self.version_minor, 999,
                                  ParserError::Version, ERR_VERSION) {
                    if handler.on_version(self.version_major, self.version_minor) {
                        skip_to_state!(State::ResponseStatusCode);
                        state_ResponseStatusCode!()
                    } else {
                        exit_ok!(State::ResponseStatusCode);
                    }
                } else {
                    State::ResponseVersionMinor
                }
            });
        }

        macro_rules! state_ResponseStatusCode {
            () => ({
                if collect_digit!(b' ', self.status_code, 999,
                                  ParserError::StatusCode, ERR_STATUS_CODE) {
                    if handler.on_status_code(self.status_code) {
                        skip_to_state!(State::ResponseStatus);
                        state_ResponseStatus!()
                    } else {
                        exit_ok!(State::ResponseStatus);
                    }
                } else {
                    State::ResponseStatusCode
                }
            });
        }

        macro_rules! state_ResponseStatus {
            () => ({
                lazy_callback!(on_status, true);

                if collect_token_space_tab_until!(b'\r', ParserError::Status, ERR_STATUS) {
                    forget!();
                    skip_to_state!(State::PreHeaders1);
                    state_PreHeaders1!()
                } else {
                    State::ResponseStatus
                }
            });
        }

        macro_rules! state_PreHeaders1 {
            () => (
                if byte == b'\n' {
                    skip_to_state!(State::PreHeaders2);
                    state_PreHeaders2!()
                } else {
                    error!(ParserError::CrlfSequence(ERR_CRLF_SEQUENCE));
                }
            );
        }

        macro_rules! state_PreHeaders2 {
            () => ({
                flags.remove(F_IN_INITIAL);
                flags.insert(F_IN_HEADERS);

                if byte == b'\r' {
                    State::Newline4
                } else {
                    replay!();

                    State::HeaderField
                }
            });
        }

        macro_rules! state_HeaderField {
            () => ({
                lazy_callback!(on_header_field, true);

                if collect_token_until!(b':', ParserError::HeaderField, ERR_HEADER_FIELD) {
                    forget!();
                    skip_to_state!(State::StripHeaderValue);
                    state_StripHeaderValue!()
                } else {
                    State::HeaderField
                }
            });
        }

        macro_rules! state_StripHeaderValue {
            () => ({
                if collect_only!(b' ', b'\t') {
                    if byte == b'"' {
                        skip_to_state!(State::QuotedHeaderValue);
                        state_QuotedHeaderValue!()
                    } else {
                        replay!();
                        skip_to_state!(State::HeaderValue);
                        state_HeaderValue!()
                    }
                } else {
                    State::StripHeaderValue
                }
            });
        };

        macro_rules! state_HeaderValue {
            () => ({
                lazy_callback!(on_header_value, true);

                if collect_non_control!() {
                    forget!();
                    replay!();

                    State::Newline1
                } else {
                    State::HeaderValue
                }
            });
        }

        macro_rules! state_QuotedHeaderValue {
            () => ({
                lazy_callback!(on_header_value, true);

                if collect_until!(b'"', b'\\', ParserError::HeaderValue, ERR_HEADER_VALUE) {
                    if flags.contains(F_QUOTE_ESCAPED) {
                        flags.remove(F_QUOTE_ESCAPED);

                        mark!();

                        State::QuotedHeaderValue
                    } else if byte == b'\\' {
                        flags.insert(F_QUOTE_ESCAPED);

                        if mark_index < stream_index - 1 {
                            forget!();

                            if !handler.on_header_value(marked_bytes!()) {
                                exit_ok!(State::QuotedHeaderValue);
                            }
                        }

                        State::QuotedHeaderValue
                    } else {
                        forget!();

                        State::Newline1
                    }
                } else {
                    State::QuotedHeaderValue
                }
            });
        }

        macro_rules! state_Newline1 {
            () => ({
                if has_bytes!(1) && b"\r\n" == peek_chunk!(2) {
                    jump!(1);
                    skip_to_state!(State::Newline3);
                    state_Newline3!()
                } else if byte == b'\r' {
                    skip_to_state!(State::Newline2);
                    state_Newline2!()
                } else {
                    error!(ParserError::CrlfSequence(ERR_CRLF_SEQUENCE));
                }
            });
        }

        macro_rules! state_Newline2 {
            () => (
                if byte == b'\n' {
                    skip_to_state!(State::Newline3);
                    state_Newline3!()
                } else {
                    error!(ParserError::CrlfSequence(ERR_CRLF_SEQUENCE));
                }
            );
        }

        macro_rules! state_Newline3 {
            () => (
                if byte == b'\r' {
                    skip_to_state!(State::Newline4);
                    state_Newline4!()
                } else if (byte == b' ' || byte == b'\t')
                && flags.bits & F_HEADERS_FINISHED.bits == F_NONE.bits {
                    // multiline header value
                    // it is optional within the HTTP spec to provide an empty space
                    // between multiline header values, but it seems to make sense, otherwise why
                    // would there be a newline in the first place?
                    if handler.on_header_value(b" ") {
                        skip_to_state!(State::StripHeaderValue);
                        state_StripHeaderValue!()
                    } else {
                        exit_ok!(State::StripHeaderValue);
                    }
                } else {
                    replay!();
                    skip_to_state!(State::HeaderField);
                    state_HeaderField!()
                }
            );
        }

        macro_rules! state_Newline4 {
            () => (
                if byte == b'\n' {
                    flags.insert(F_HEADERS_FINISHED);
                    flags.remove(F_IN_HEADERS);

                    if handler.on_headers_finished() {
                        skip_to_state!(State::Body);
                        state_Body!()
                    } else {
                        exit_ok!(State::Body);
                    }
                } else {
                    error!(ParserError::CrlfSequence(ERR_CRLF_SEQUENCE));
                }
            );
        }

        macro_rules! state_Body {
            () => ({
                replay!();

                match handler.get_transfer_encoding() {
                    TransferEncoding::None => {
                        match handler.get_content_type() {
                            ContentType::None | ContentType::Other(_) => {
                                if let ContentLength::Specified(length) = handler.get_content_length() {
                                    remaining_byte_count = length;

                                    skip_to_state!(State::BodyContent);
                                    state_BodyContent!()
                                } else {
                                    error!(ParserError::MissingContentLength(ERR_MISSING_CONTENT_LENGTH));
                                }
                            },
                            ContentType::Multipart(_) => {
                                skip_to_state!(State::MultipartStartingBoundary);
                                state_MultipartStartingBoundary!()
                            },
                            ContentType::UrlEncoded(_) => {
                                skip_to_state!(State::UrlEncoded);
                                state_UrlEncoded!()
                            }
                        }
                    },
                    TransferEncoding::Chunked => {
                        overflow_index       = 0;
                        remaining_byte_count = 0;

                        skip_to_state!(State::ChunkSize);
                        state_ChunkSize!()
                    }
                }
            });
        }

        macro_rules! state_BodyContent {
            () => (
                State::BodyContent
            );
        }

        macro_rules! state_ChunkSize {
            () => ({
                if collect_hex_digit!(b'\r', b';', CFG_MAX_CHUNK_SIZE_LENGTH,
                                      ParserError::MaxChunkSizeLength, ERR_MAX_CHUNK_SIZE_LENGTH,
                                      ParserError::ChunkSize, ERR_CHUNK_SIZE) {
                    if byte == b'\r' {
                        if handler.on_chunk_size(remaining_byte_count) {
                            skip_to_state!(State::ChunkNewline);
                            state_ChunkNewline!()
                        } else {
                            State::ChunkNewline
                        }
                    } else {
                        overflow_index = 0;

                        if handler.on_chunk_size(remaining_byte_count) {
                            skip_to_state!(State::ChunkExtension);
                            state_ChunkExtension!()
                        } else {
                            State::ChunkExtension
                        }
                    }
                } else {
                    State::ChunkSize
                }
            });
        }

        macro_rules! state_ChunkExtension {
            () => ({
                lazy_callback!(on_chunk_extension, true);

                if collect_token_until_overflow!(b'\r', CFG_MAX_CHUNK_EXTENSION_LENGTH,
                                                 ParserError::MaxChunkExtensionLength,
                                                 ERR_MAX_CHUNK_EXTENSION_LENGTH,
                                                 ParserError::ChunkExtension, ERR_CHUNK_EXTENSION) {
                    forget!();

                    skip_to_state!(State::ChunkNewline);
                    state_ChunkNewline!()
                } else {
                    State::ChunkExtension
                }
            });
        }

        macro_rules! state_ChunkNewline {
            () => (
                if byte == b'\n' {
                    skip_to_state!(State::ChunkData);
                    state_ChunkData!()
                } else {
                    error!(ParserError::CrlfSequence(ERR_CRLF_SEQUENCE));
                }
            );
        }

        macro_rules! state_ChunkData {
            () => (
                State::ChunkData
            );
        }

        macro_rules! state_MultipartStartingBoundary {
            () => (
                State::MultipartStartingBoundary
            );
        }

        macro_rules! state_MultipartData {
            () => (
                State::MultipartData
            );
        }

        macro_rules! state_MultipartEndingBoundary {
            () => (
                State::MultipartEndingBoundary
            );
        }

        macro_rules! state_MultipartFinished {
            () => (
                State::MultipartFinished
            );
        }

        macro_rules! state_UrlEncoded {
            () => (
                State::UrlEncoded
            );
        }

        // -----------------------------------------------------------------------------------------

        if flags.contains(F_REQUEST) {
            loop {
                top_of_loop!();

                state = if flags.contains(F_IN_HEADERS) {
                    match state {
                        State::HeaderField       => state_HeaderField!(),
                        State::HeaderValue       => state_HeaderValue!(),
                        State::Newline1          => state_Newline1!(),
                        State::Newline2          => state_Newline2!(),
                        State::Newline3          => state_Newline3!(),
                        State::Newline4          => state_Newline4!(),
                        State::QuotedHeaderValue => state_QuotedHeaderValue!(),
                        State::StripHeaderValue  => state_StripHeaderValue!(),

                        _ => {
                            error!(ParserError::Dead(ERR_DEAD));
                        }
                    }
                } else if flags.contains(F_IN_INITIAL) {
                    match state {
                        State::RequestMethod         => state_RequestMethod!(),
                        State::RequestUrl            => state_RequestUrl!(),
                        State::RequestHttp1          => state_RequestHttp1!(),
                        State::RequestVersionMajor   => state_RequestVersionMajor!(),
                        State::RequestVersionMinor   => state_RequestVersionMinor!(),
                        State::PreHeaders1           => state_PreHeaders1!(),
                        State::PreHeaders2           => state_PreHeaders2!(),
                        State::RequestHttp2          => state_RequestHttp2!(),
                        State::RequestHttp3          => state_RequestHttp3!(),
                        State::RequestHttp4          => state_RequestHttp4!(),
                        State::RequestHttp5          => state_RequestHttp5!(),

                        _ => {
                            error!(ParserError::Dead(ERR_DEAD));
                        }
                    }
                } else {
                    match state {
                        State::Body                      => state_Body!(),
                        State::BodyContent               => state_BodyContent!(),
                        State::ChunkSize                 => state_ChunkSize!(),
                        State::ChunkExtension            => state_ChunkExtension!(),
                        State::ChunkNewline              => state_ChunkNewline!(),
                        State::ChunkData                 => state_ChunkData!(),
                        State::MultipartStartingBoundary => state_MultipartStartingBoundary!(),
                        State::MultipartData             => state_MultipartData!(),
                        State::MultipartEndingBoundary   => state_MultipartEndingBoundary!(),
                        State::MultipartFinished         => state_MultipartFinished!(),
                        State::UrlEncoded                => state_UrlEncoded!(),

                        _ => {
                            error!(ParserError::Dead(ERR_DEAD));
                        }
                    }
                }
            }
        } else {
            loop {
                top_of_loop!();

                state = if flags.contains(F_IN_HEADERS) {
                    match state {
                        State::HeaderField       => state_HeaderField!(),
                        State::HeaderValue       => state_HeaderValue!(),
                        State::Newline1          => state_Newline1!(),
                        State::Newline2          => state_Newline2!(),
                        State::Newline3          => state_Newline3!(),
                        State::Newline4          => state_Newline4!(),
                        State::QuotedHeaderValue => state_QuotedHeaderValue!(),
                        State::StripHeaderValue  => state_StripHeaderValue!(),

                        _ => {
                            error!(ParserError::Dead(ERR_DEAD));
                        }
                    }
                } else if flags.contains(F_IN_INITIAL) {
                    match state {
                        State::ResponseHttp1        => state_ResponseHttp1!(),
                        State::ResponseVersionMajor => state_ResponseVersionMajor!(),
                        State::ResponseVersionMinor => state_ResponseVersionMinor!(),
                        State::ResponseStatusCode   => state_ResponseStatusCode!(),
                        State::ResponseStatus       => state_ResponseStatus!(),
                        State::PreHeaders1          => state_PreHeaders1!(),
                        State::PreHeaders2          => state_PreHeaders2!(),
                        State::ResponseHttp2        => state_ResponseHttp2!(),
                        State::ResponseHttp3        => state_ResponseHttp3!(),
                        State::ResponseHttp4        => state_ResponseHttp4!(),
                        State::ResponseHttp5        => state_ResponseHttp5!(),

                        _ => {
                            error!(ParserError::Dead(ERR_DEAD));
                        }
                    }
                } else {
                    match state {
                        State::Body                      => state_Body!(),
                        State::BodyContent               => state_BodyContent!(),
                        State::ChunkSize                 => state_ChunkSize!(),
                        State::ChunkExtension            => state_ChunkExtension!(),
                        State::ChunkNewline              => state_ChunkNewline!(),
                        State::ChunkData                 => state_ChunkData!(),
                        State::MultipartStartingBoundary => state_MultipartStartingBoundary!(),
                        State::MultipartData             => state_MultipartData!(),
                        State::MultipartEndingBoundary   => state_MultipartEndingBoundary!(),
                        State::MultipartFinished         => state_MultipartFinished!(),
                        State::UrlEncoded                => state_UrlEncoded!(),

                        _ => {
                            error!(ParserError::Dead(ERR_DEAD));
                        }
                    }
                }
            }
        }
    }

    /// Reset the parser to its initial state.
    pub fn reset(&mut self) {
        self.byte_count           = 0;
        self.callback             = Callback::None;
        self.content_type         = ContentType::None;
        self.flags                = if self.flags.contains(F_REQUEST) {
                                        F_IN_INITIAL | F_REQUEST
                                    } else {
                                        F_IN_INITIAL
                                    };
        self.overflow_index       = 0;
        self.remaining_byte_count = 0;
        self.status_code          = 0;
        self.version_major        = 0;
        self.version_minor        = 0;

        self.state = if self.flags.contains(F_REQUEST) {
            State::RequestMethod
        } else {
            State::ResponseHttp1
        }
    }
}
