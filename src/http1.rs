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
use url::{ ParamError,
           ParamHandler,
           parse_query_string };

/// Maximum content byte count to process before returning `ParserError::MaxContentLength`.
pub const CFG_MAX_CONTENT_LENGTH: u64 = !0u64;

/// Maximum chunk extension byte count to process before returning
/// `ParserError::MaxChunkExtensionLength`.
pub const CFG_MAX_CHUNK_EXTENSION_LENGTH: u8 = 255;

/// Maximum chunk size byte count to process before returning `ParserError::MaxChunkSizeLength`.
pub const CFG_MAX_CHUNK_SIZE_LENGTH: u8 = 16;

/// Maximum headers byte count to process before returning `ParserError::MaxHeadersLength`.
pub const CFG_MAX_HEADERS_LENGTH: u32 = 80 * 1024;

/// Maximum multipart boundary byte count to process before returning
/// `ParserError::MaxMultipartBoundaryLength`.
pub const CFG_MAX_MULTIPART_BOUNDARY_LENGTH: u8 = 70;

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

/// Maximum conent length has been met.
pub const ERR_MAX_CONTENT_LENGTH: &'static str = "Maximum content length";

/// Maximum chunk extension length has been met.
pub const ERR_MAX_CHUNK_EXTENSION_LENGTH: &'static str = "Maximum chunk extension length";

/// Maximum chunk size length has been met.
pub const ERR_MAX_CHUNK_SIZE_LENGTH: &'static str = "Maximum chunk size length";

/// Maximum headers length has been met.
pub const ERR_MAX_HEADERS_LENGTH: &'static str = "Maximum headers length";

/// Maximum multipart boundary length.
pub const ERR_MAX_MULTIPART_BOUNDARY_LENGTH: &'static str = "Maximum multipart boundary length";

/// Maximul URL encoded data length.
pub const ERR_MAX_URL_ENCODED_DATA_LENGTH: &'static str = "Maximum URL encoded data length";

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

#[allow(dead_code)]
enum Callback<T> {
    None,
    Data(fn(&mut T, &[u8]) -> bool),
    DataLength(fn(&mut T, &[u8]) -> bool)
}

/// Connection.
#[derive(Clone,Copy,PartialEq)]
#[repr(u8)]
pub enum Connection {
    None,
    Close,
    KeepAlive,
    Upgrade
}

/// Content length.
#[derive(Clone,Copy,PartialEq)]
pub enum ContentLength {
    None,
    Specified(u64)
}

/// Content type.
pub enum ContentType {
    None,
    Multipart(Vec<u8>),
    UrlEncoded,
    Other(Vec<u8>),
}

/// Parser error messages.
#[derive(Clone,Copy,PartialEq)]
pub enum ParserError {
    /// Invalid chunk extension.
    ChunkExtension(&'static str, u8),

    /// Invalid chunk size.
    ChunkSize(&'static str, u8),

    /// Invalid CRLF sequence.
    CrlfSequence(&'static str),

    /// Parsing has failed, but `Parser::parse()` is executed again.
    Dead(&'static str),

    /// Invalid header field.
    HeaderField(&'static str, u8),

    /// Invalid header value.
    HeaderValue(&'static str, u8),

    /// Maximum chunk extension length has been met.
    MaxChunkExtensionLength(&'static str, u8),

    /// Maximum chunk size has been met.
    MaxChunkSizeLength(&'static str, u8),

    /// Maximum content length has been met.
    MaxContentLength(&'static str, u8),

    /// Maximum headers length has been met.
    MaxHeadersLength(&'static str, u32),

    /// Maximum URL encoded data length.
    MaxUrlEncodedDataLength(&'static str),

    /// Missing an expected Content-Length header.
    MissingContentLength(&'static str),

    /// Invalid request method.
    Method(&'static str, u8),

    /// Invalid multipart boundary.
    MultipartBoundary(&'static str, u8),

    /// Invalid status.
    Status(&'static str, u8),

    /// Invalid status code.
    StatusCode(&'static str),

    /// Invalid URL character.
    Url(&'static str, u8),

    /// Invalid URL encoded field.
    UrlEncodedField(&'static str, u8),

    /// Invalid URL encoded value.
    UrlEncodedValue(&'static str, u8),

    /// Invalid HTTP version.
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

    /// Parser has finished successfully.
    Done,

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

    /// Unparsable content.
    Content,

    /// Parsing chunk size.
    ChunkSize,

    /// Parsing chunk extension.
    ChunkExtension,

    /// CRLF after chunk size.
    ChunkSizeNewline1,
    ChunkSizeNewline2,

    /// Parsing chunk data.
    ChunkData,

    /// CRLF after chunk data.
    ChunkDataNewline1,
    ChunkDataNewline2,

    /// Parsing hyphen's before and after multipart boundary.
    MultipartHyphen1,
    MultipartHyphen2,

    /// Parsing multipart boundary.
    MultipartBoundary,

    /// CRLF after boundary.
    MultipartNewline1,
    MultipartNewline2,

    /// Parsing multipart data.
    MultipartData,

    /// Parsing URL encoded field.
    UrlEncodedField,

    /// Parsing URL encoded field hex/plus/ampersand sequence.
    UrlEncodedFieldHex,

    /// Parsing URL encoded value.
    UrlEncodedValue,

    /// Parsing URL encoded value hex/plus/ampersand sequence.
    UrlEncodedValueHex,

    /// CRLF after URL encoded data.
    UrlEncodedNewline1,
    UrlEncodedNewline2
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
#[derive(Clone,PartialEq)]
#[repr(u8)]
pub enum TransferEncoding {
    None,
    Chunked,
    Compress,
    Deflate,
    Gzip,
    Other(String)
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

    /// Callback that is executed when multipart data has been located.
    ///
    /// This may be executed multiple times in order to supply the entire piece of data.
    fn on_multipart_data(&mut self, method: &[u8]) -> bool {
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

        // Forget most recent byte.
        const B_FORGET = 1 << 0
    }
}

// Flags used to track request/response/state details.
bitflags! {
    flags Flag: u8 {
        // No flags.
        const F_NONE = 0,

        // Parsing chunk data.
        const F_CHUNK_DATA = 1 << 0,

        // Parsing data that needs to check against content length.
        const F_CONTENT_LENGTH = 1 << 1,

        // Parsing initial.
        const F_IN_INITIAL = 1 << 2,

        // Parsing headers.
        const F_IN_HEADERS = 1 << 3,

        // Finished parsing headers.
        const F_HEADERS_FINISHED = 1 << 4,

        // Quoted header value has an escape character.
        const F_QUOTE_ESCAPED = 1 << 5,

        // Parsing multipart data.
        const F_MULTIPART_DATA = 1 << 6,

        // Indicates the stream contains request data rather than response.
        const F_REQUEST = 1 << 7
    }
}

pub struct Parser<T: HttpHandler + ParamHandler> {
    // Total byte count processed for headers, and body.
    // Once the headers are finished processing, this is reset to 0 to track the body length.
    byte_count: usize,

    // Current callback that is handled at the top of each byte loop.
    callback: Callback<T>,

    // Content length used to track amount of bytes left to process.
    content_length: u64,

    // Content type.
    content_type: ContentType,

    // The request/response flags.
    flags: Flag,

    // Maximum header byte count to process before we assume it's a DoS stream.
    max_headers_length: u32,

    // Index used for overflow detection.
    overflow_index: u8,

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

impl<T: HttpHandler + ParamHandler> Parser<T> {
    /// Create a new `Parser` with a maximum headers length of `80 * 1024`.
    pub fn new(stream_type: StreamType) -> Parser<T> {
        Parser{ byte_count:           0,
                callback:             Callback::None,
                content_length:       0,
                content_type:         ContentType::None,
                flags:                if stream_type == StreamType::Request {
                                          F_IN_INITIAL | F_REQUEST
                                      } else {
                                          F_IN_INITIAL
                                      },
                max_headers_length:   CFG_MAX_HEADERS_LENGTH,
                overflow_index:       0,
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
    pub fn with_settings(stream_type: StreamType, max_headers_length: u32) -> Parser<T> {
        Parser{ byte_count:           0,
                callback:             Callback::None,
                content_length:       0,
                content_type:         ContentType::None,
                flags:                if stream_type == StreamType::Request {
                                          F_IN_INITIAL | F_REQUEST
                                      } else {
                                          F_IN_INITIAL
                                      },
                max_headers_length:   max_headers_length,
                overflow_index:       0,
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
    pub fn parse(&mut self, handler: &mut T, mut stream: &[u8]) -> Result<Success, ParserError> {
        // current byte
        let mut byte: u8 = 0;

        // byte flags
        let mut byte_flags: ByteFlag = B_NONE;

        // callback to execute
        let mut callback = match self.callback {
            Callback::Data(x)       => Callback::Data(x),
            Callback::DataLength(x) => Callback::DataLength(x),
            Callback::None          => Callback::None
        };

        // content length
        let mut content_length = self.content_length;

        // content type
        let mut content_type = ContentType::None;

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

        // current state
        let mut state = self.state;

        // stream index we're processing
        let mut stream_index: usize = 0;

        // stream length
        let stream_length = stream.len();

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

        // check content length is equal to or less than byte count
        macro_rules! check_content_length {
            () => (
                byte_count < content_length
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
            ($byte1:expr, $byte2:expr, $digit:expr, $max:expr, $overflow_error:path,
             $overflow_error_msg:expr, $error:path, $error_msg:expr) => (
                collect_base!({
                    if is_hex!(byte) {
                        if $max == overflow_index {
                            error!($overflow_error($overflow_error_msg, byte));
                        }

                        inc_overflow!();

                        $digit <<= 4;

                        match hex_to_byte(&[byte]) {
                            Some(byte) => {
                                $digit += byte as u64;
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

        // collect remaining byte count
        macro_rules! collect_remaining_unsafe {
            () => ({
                stream_index -= 1;

                let slice = if content_length <= (stream.len() - stream_index) as u64 {
                    &stream[stream_index..stream_index + content_length as usize]
                } else {
                    &stream[stream_index..stream.len()]
                };

                content_length -= slice.len() as u64;
                stream_index   += slice.len();

                content_length == 0
            });
        }

        // collect non-control characters until a certain byte is found
        macro_rules! collect_until {
            ($byte1:expr, $byte2:expr, $byte3:expr, $byte4:expr, $byte5:expr,
             $error:path, $error_msg:expr) => (
                collect_base!({
                    if $byte1 == byte
                    || $byte2 == byte
                    || $byte3 == byte
                    || $byte4 == byte
                    || $byte5 == byte {
                        true
                    } else if !is_ascii!(byte) || is_control!(byte) {
                        error!($error($error_msg, byte));
                    } else {
                        false
                    }
                })
            );

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
            ($error:expr) => ({
                self.state = State::Dead;

                return Err($error);
            });
        }

        // save state and exit with callback status
        macro_rules! exit_callback {
            () => ({
                save!();

                return Ok(Success::Callback(stream_length - (stream.len() - stream_index)));
            });

            ($state:expr) => ({
                save!($state);

                return Ok(Success::Callback(stream_length - (stream.len() - stream_index)));
            });
        }

        // save state and exit with eof status
        macro_rules! exit_eof {
            () => ({
                save!();

                return Ok(Success::Eof(stream_length - (stream.len() - stream_index)));
            });
        }

        // save state and exit with finished status
        macro_rules! exit_finished {
            () => ({
                save!(State::Done);

                return Ok(Success::Finished(stream_length - (stream.len() - stream_index)));
            });
        }

        // forget one byte when marked_bytes!() is called
        // do not use this with replay!(), only use replay!(), as it serves both purposes
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

        // increment overflow index
        macro_rules! inc_overflow {
            () => (
                overflow_index += 1
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
        macro_rules! callback_data {
            // callback with data
            ($function:ident) => (
                callback = Callback::Data(T::$function);
            );
        }

        // save a callback to be executed lazily by the parser, followed by a content length check
        macro_rules! callback_data_length {
            ($function:ident) => (
                callback = Callback::DataLength(T::$function);
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

            ($length:expr) => (
                &stream[mark_index..mark_index + $length]
            );
        }

        // move the stream forward and reset the byte count
        macro_rules! move_stream {
            () => ({
                self.byte_count = 0;
                stream          = &stream[stream_index-1..stream.len()];
                stream_index    = 0;

                next!();
            });
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
                stream_index -= 1
            );
        }

        // reset content length
        macro_rules! reset_content_length {
            () => (
                content_length = 0
            );
        }

        // reset overflow index
        macro_rules! reset_overflow {
            () => (
                overflow_index = 0
            );
        }

        // save parser details
        macro_rules! save {
            () => (
                save!(state)
            );

            ($state:expr) => (
                self.byte_count     += stream_index;
                self.callback        = callback;
                self.content_length  = content_length;
                self.content_type    = content_type;
                self.flags           = flags;
                self.overflow_index  = overflow_index;
                self.state           = $state;
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
                                exit_callback!();
                            }
                        },
                        Callback::DataLength(x) => {
                            callback = Callback::None;

                            if !x(handler, marked_bytes!()) {
                                exit_callback!();
                            }
                        },
                        Callback::None => {
                        }
                    }
                } else if is_eof!() {
                    match callback {
                        Callback::Data(x) => {
                            if !x(handler, marked_bytes!()) {
                                exit_callback!();
                            }
                        },
                        Callback::DataLength(x) => {
                            if !x(handler, marked_bytes!()) {
                                exit_callback!();
                            }
                        },
                        Callback::None => {
                        }
                    }
                }

                next!();

                if old_state != state {
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
                callback_data!(on_method);

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
                    exit_callback!(State::RequestUrl);
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
                callback_data!(on_url);

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
                        exit_callback!(State::PreHeaders1);
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
                        exit_callback!(State::ResponseStatusCode);
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
                        exit_callback!(State::ResponseStatus);
                    }
                } else {
                    State::ResponseStatusCode
                }
            });
        }

        macro_rules! state_ResponseStatus {
            () => ({
                callback_data!(on_status);

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
                callback_data!(on_header_field);

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
                callback_data!(on_header_value);

                if collect_non_control!() {
                    replay!();

                    State::Newline1
                } else {
                    State::HeaderValue
                }
            });
        }

        macro_rules! state_QuotedHeaderValue {
            () => ({
                callback_data!(on_header_value);

                if collect_until!(b'"', b'\\', ParserError::HeaderValue, ERR_HEADER_VALUE) {
                    if flags.bits & F_QUOTE_ESCAPED.bits == F_QUOTE_ESCAPED.bits {
                        flags.remove(F_QUOTE_ESCAPED);

                        mark!();

                        State::QuotedHeaderValue
                    } else if byte == b'\\' {
                        flags.insert(F_QUOTE_ESCAPED);

                        if mark_index < stream_index - 1 {
                            forget!();

                            if !handler.on_header_value(marked_bytes!()) {
                                exit_callback!();
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
                        exit_callback!(State::StripHeaderValue);
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
                        if flags.bits & F_CHUNK_DATA.bits == F_CHUNK_DATA.bits {
                            handler.on_finished();

                            exit_finished!();
                        }

                        State::Body
                    } else if flags.bits & F_CHUNK_DATA.bits == F_CHUNK_DATA.bits {
                        handler.on_finished();

                        exit_finished!();
                    } else {
                        exit_callback!(State::Body);
                    }
                } else {
                    error!(ParserError::CrlfSequence(ERR_CRLF_SEQUENCE));
                }
            );
        }

        macro_rules! state_Body {
            () => ({
                move_stream!();
                replay!();

                if handler.get_transfer_encoding() == TransferEncoding::Chunked {
                    reset_overflow!();
                    reset_content_length!();

                    flags.insert(F_CHUNK_DATA);

                    State::ChunkSize
                } else {
                    content_type = handler.get_content_type();

                    match content_type {
                        ContentType::None | ContentType::Other(_) => {
                            if let ContentLength::Specified(length) = handler.get_content_length() {
                                content_length = length;

                                State::Content
                            } else {
                                error!(ParserError::MissingContentLength(ERR_MISSING_CONTENT_LENGTH));
                            }
                        },
                        ContentType::Multipart(_) => {
                            State::MultipartHyphen1
                        },
                        ContentType::UrlEncoded => {
                            if let ContentLength::Specified(length) = handler.get_content_length() {
                                flags.insert(F_CONTENT_LENGTH);

                                content_length = length;

                                State::UrlEncodedField
                            } else {
                                error!(ParserError::MissingContentLength(ERR_MISSING_CONTENT_LENGTH));
                            }
                        }
                    }
                }
            });
        }

        macro_rules! state_Content {
            () => (
                State::Content
            );
        }

        macro_rules! state_ChunkSize {
            () => (
                if collect_hex_digit!(b'\r', b';', content_length, CFG_MAX_CHUNK_SIZE_LENGTH,
                                      ParserError::MaxChunkSizeLength, ERR_MAX_CHUNK_SIZE_LENGTH,
                                      ParserError::ChunkSize, ERR_CHUNK_SIZE) {
                    if content_length == 0 {
                        flags.insert(F_IN_HEADERS);
                        flags.remove(F_HEADERS_FINISHED);

                        if handler.on_chunk_size(content_length) {
                            replay!();

                            State::Newline1
                        } else {
                            exit_callback!(State::Newline2);
                        }
                    } else if byte == b'\r' {
                        if handler.on_chunk_size(content_length) {
                            replay!();

                            skip_to_state!(State::ChunkSizeNewline1);
                            state_ChunkSizeNewline1!()
                        } else {
                            exit_callback!(State::ChunkSizeNewline2);
                        }
                    } else {
                        reset_overflow!();

                        if handler.on_chunk_size(content_length) {
                            skip_to_state!(State::ChunkExtension);
                            state_ChunkExtension!()
                        } else {
                            exit_callback!(State::ChunkExtension);
                        }
                    }
                } else {
                    State::ChunkSize
                }
            );
        }

        macro_rules! state_ChunkExtension {
            () => ({
                callback_data!(on_chunk_extension);

                if collect_token_until_overflow!(b'\r', CFG_MAX_CHUNK_EXTENSION_LENGTH,
                                                 ParserError::MaxChunkExtensionLength,
                                                 ERR_MAX_CHUNK_EXTENSION_LENGTH,
                                                 ParserError::ChunkExtension, ERR_CHUNK_EXTENSION) {
                    replay!();

                    skip_to_state!(State::ChunkSizeNewline1);
                    state_ChunkSizeNewline1!()
                } else {
                    State::ChunkExtension
                }
            });
        }

        macro_rules! state_ChunkSizeNewline1 {
            () => (
                if has_bytes!(1) && b"\r\n" == peek_chunk!(2) {
                    jump!(1);
                    skip_to_state!(State::ChunkData);
                    state_ChunkData!()
                } else if byte == b'\r' {
                    skip_to_state!(State::ChunkSizeNewline2);
                    state_ChunkSizeNewline2!()
                } else {
                    error!(ParserError::CrlfSequence(ERR_CRLF_SEQUENCE));
                }
            );
        }

        macro_rules! state_ChunkSizeNewline2 {
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
            () => ({
                callback_data!(on_chunk_data);

                if collect_remaining_unsafe!() {
                    skip_to_state!(State::ChunkDataNewline1);
                    state_ChunkDataNewline1!()
                } else {
                    State::ChunkData
                }
            });
        }

        macro_rules! state_ChunkDataNewline1 {
            () => (
                if byte == b'\r' {
                    skip_to_state!(State::ChunkDataNewline2);
                    state_ChunkDataNewline2!()
                } else {
                    error!(ParserError::CrlfSequence(ERR_CRLF_SEQUENCE));
                }
            );
        }

        macro_rules! state_ChunkDataNewline2 {
            () => (
                if byte == b'\n' {
                    reset_overflow!();
                    reset_content_length!();

                    State::ChunkSize
                } else {
                    error!(ParserError::CrlfSequence(ERR_CRLF_SEQUENCE));
                }
            );
        }

        macro_rules! state_MultipartHyphen1 {
            () => (
                State::MultipartHyphen1
            );
        }

        macro_rules! state_MultipartHyphen2 {
            () => (
                State::MultipartHyphen2
            );
        }

        macro_rules! state_MultipartBoundary {
            () => (
                State::MultipartBoundary
            );
        }

        macro_rules! state_MultipartData {
            () => (
                State::MultipartData
            );
        }

        macro_rules! state_MultipartNewline1 {
            () => (
                State::MultipartNewline1
            );
        }

        macro_rules! state_MultipartNewline2 {
            () => (
                State::MultipartNewline2
            );
        }

        macro_rules! state_UrlEncodedField {
            () => ({
                callback_data!(on_param_field);

                if collect_until!(b'\r', b'=', b'%', b'&', b'+', ParserError::UrlEncodedField,
                                         ERR_URL_ENCODED_FIELD) {
                    if byte == b'=' {
                        forget!();

                        skip_to_state!(State::UrlEncodedValue);
                        state_UrlEncodedValue!()
                    } else if byte == b'%' {
                        replay!();

                        skip_to_state!(State::UrlEncodedFieldHex);
                        state_UrlEncodedFieldHex!()
                    } else if byte == b'+' || byte == b'&' {
                        replay!();

                        skip_to_state!(State::UrlEncodedFieldHex);
                        state_UrlEncodedFieldHex!()
                    } else {
                        replay!();

                        skip_to_state!(State::UrlEncodedNewline1);
                        state_UrlEncodedNewline1!()
                    }
                } else {
                    State::UrlEncodedField
                }
            });
        }

        macro_rules! state_UrlEncodedFieldHex {
            () => (
                State::UrlEncodedFieldHex
                /*
                if byte == b'%' {
                    if !has_bytes!(2) {
                        error!(ParserError::Eof(stream_index-1));
                    }

                    if content_length < 2 {
                        for i in 0..content_length {
                            next!();
                        }
                    } else {
                        next!();
                        mark!();
                        next!();
                        inc_remaining!();
                        inc_remaining!();

                        match hex_to_byte(marked_bytes!()) {
                            Some(byte) => {
                                if !handler.on_param_field(&[byte]) {
                                    exit_callback_remaining!();
                                }
                            },
                            _ => {
                                error!(ParserError::UrlEncodedField(ERR_URL_ENCODED_FIELD, byte));
                            }
                        }

                        State::UrlEncodedField
                    }
                } else if byte == b'+' {
                    if !handler.on_param_field(b" ") {
                        replay!();

                        exit_callback_remaining!();
                    }

                    State::UrlEncodedField
                } else {
                    // param with no value
                    if !handler.on_param_value(b" ") {
                        replay!();

                        exit_callback_remaining!();
                    }

                    State::UrlEncodedField
                }
                */
            );
        }

        macro_rules! state_UrlEncodedValue {
            () => (
                State::UrlEncodedValue
            );
        }

        macro_rules! state_UrlEncodedValueHex {
            () => (
                State::UrlEncodedValueHex
            );
        }

        macro_rules! state_UrlEncodedNewline1 {
            () => (
                if byte == b'\r' {
                    skip_to_state!(State::UrlEncodedNewline2);
                    state_UrlEncodedNewline2!()
                } else {
                    error!(ParserError::CrlfSequence(ERR_CRLF_SEQUENCE));
                }
            );
        }

        macro_rules! state_UrlEncodedNewline2 {
            () => (
                if byte == b'\n' {
                    exit_finished!();
                } else {
                    error!(ParserError::CrlfSequence(ERR_CRLF_SEQUENCE));
                }
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
                        State::UrlEncodedField    => state_UrlEncodedField!(),
                        State::UrlEncodedFieldHex => state_UrlEncodedFieldHex!(),
                        State::UrlEncodedValue    => state_UrlEncodedValue!(),
                        State::UrlEncodedValueHex => state_UrlEncodedValueHex!(),
                        State::UrlEncodedNewline1 => state_UrlEncodedNewline1!(),
                        State::UrlEncodedNewline2 => state_UrlEncodedNewline2!(),
                        State::Body               => state_Body!(),
                        State::Content            => state_Content!(),
                        State::ChunkSize          => state_ChunkSize!(),
                        State::ChunkExtension     => state_ChunkExtension!(),
                        State::ChunkSizeNewline1  => state_ChunkSizeNewline1!(),
                        State::ChunkSizeNewline2  => state_ChunkSizeNewline2!(),
                        State::ChunkData          => state_ChunkData!(),
                        State::ChunkDataNewline1  => state_ChunkDataNewline1!(),
                        State::ChunkDataNewline2  => state_ChunkDataNewline2!(),
                        State::MultipartHyphen1   => state_MultipartHyphen1!(),
                        State::MultipartHyphen2   => state_MultipartHyphen2!(),
                        State::MultipartBoundary  => state_MultipartBoundary!(),
                        State::MultipartNewline1  => state_MultipartNewline1!(),
                        State::MultipartNewline2  => state_MultipartNewline2!(),
                        State::MultipartData      => state_MultipartData!(),

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
                        State::UrlEncodedField    => state_UrlEncodedField!(),
                        State::UrlEncodedFieldHex => state_UrlEncodedFieldHex!(),
                        State::UrlEncodedValue    => state_UrlEncodedValue!(),
                        State::UrlEncodedValueHex => state_UrlEncodedValueHex!(),
                        State::UrlEncodedNewline1 => state_UrlEncodedNewline1!(),
                        State::UrlEncodedNewline2 => state_UrlEncodedNewline2!(),
                        State::Body               => state_Body!(),
                        State::Content            => state_Content!(),
                        State::ChunkSize          => state_ChunkSize!(),
                        State::ChunkExtension     => state_ChunkExtension!(),
                        State::ChunkSizeNewline1  => state_ChunkSizeNewline1!(),
                        State::ChunkSizeNewline2  => state_ChunkSizeNewline2!(),
                        State::ChunkData          => state_ChunkData!(),
                        State::ChunkDataNewline1  => state_ChunkDataNewline1!(),
                        State::ChunkDataNewline2  => state_ChunkDataNewline2!(),
                        State::MultipartHyphen1   => state_MultipartHyphen1!(),
                        State::MultipartHyphen2   => state_MultipartHyphen2!(),
                        State::MultipartBoundary  => state_MultipartBoundary!(),
                        State::MultipartNewline1  => state_MultipartNewline1!(),
                        State::MultipartNewline2  => state_MultipartNewline2!(),
                        State::MultipartData      => state_MultipartData!(),

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
        self.content_length       = 0;
        self.content_type         = ContentType::None;
        self.flags                = if self.flags.contains(F_REQUEST) {
                                        F_IN_INITIAL | F_REQUEST
                                    } else {
                                        F_IN_INITIAL
                                    };
        self.overflow_index       = 0;
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
