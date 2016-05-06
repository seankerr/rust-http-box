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

use std::str;

/// Maximum chunk extension byte count to process before returning
/// `ParserError::MaxChunkExtensionLength`.
pub const CFG_MAX_CHUNK_EXTENSION_LENGTH: u8 = 255;

/// Maximum chunk size byte count to process before returning `ParserError::MaxChunkSizeLength`.
pub const CFG_MAX_CHUNK_SIZE_LENGTH: u8 = 16;

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

/// Maximum chunk extension length has been met.
pub const ERR_MAX_CHUNK_EXTENSION_LENGTH: &'static str = "Maximum chunk extension length";

/// Maximum chunk size length has been met.
pub const ERR_MAX_CHUNK_SIZE_LENGTH: &'static str = "Maximum chunk size length";

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
    flags Flag: u8 {
        // No flags.
        const F_NONE = 0,

        // Parsing chunk data.
        const F_CHUNK_DATA = 1 << 0,

        // Parsing data that needs to check against content length.
        const F_CONTENT_LENGTH = 1 << 1,

        // Parsing multipart data.
        const F_MULTIPART_DATA = 1 << 2
    }
}

// -------------------------------------------------------------------------------------------------

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
    CrlfSequence(&'static str, u8),

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
    /// Exit the parser loop.
    Exit(Success),

    /// Continue the parser loop.
    Continue,

    /// Shift the stream slice over a specified length.
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

    /// Parser has finished successfully.
    Finished,

    // ---------------------------------------------------------------------------------------------
    // REQUEST
    // ---------------------------------------------------------------------------------------------

    /// Stripping space before method.
    StripRequestMethod,

    /// Parsing request method.
    RequestMethod,

    /// Stripping space before URL.
    StripRequestUrl,

    /// Determining if URL starts with a scheme, or is an absolute path
    RequestUrl,

    /// Stripping space before request HTTP version.
    StripRequestHttp,

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

    /// Stripping space before response HTTP version.
    StripResponseHttp,

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

    /// Stripping space before response status code.
    StripResponseStatusCode,

    /// Parsing response status code.
    ResponseStatusCode,

    /// Stripping space before response status.
    StripResponseStatus,

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

    /// Stripping space before header field.
    StripHeaderField,

    /// Parsing first byte of header field.
    FirstHeaderField,

    /// Parsing header field.
    HeaderField,

    /// Stripping space before header value.
    StripHeaderValue,

    /// Parsing header value.
    HeaderValue,

    /// Parsing header quoted value.
    QuotedHeaderValue,

    /// Parsing header quoted escaped value.
    QuotedEscapedHeaderValue,

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

    /// Parsing URL encoded field ampersand.
    UrlEncodedFieldAmpersand,

    /// Parsing URL encoded field hex sequence.
    UrlEncodedFieldHex,

    /// Parsing URL encoded field plus.
    UrlEncodedFieldPlus,

    /// Parsing URL encoded value.
    UrlEncodedValue,

    /// Parsing URL encoded value hex sequence.
    UrlEncodedValueHex,

    /// Parsing URL encoded value plus.
    UrlEncodedValuePlus,

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

/// Parser state function type.
pub type StateFunction<T> = fn(&mut Parser<T>, &mut ParserContext<T>)
    -> Result<ParserValue, ParserError>;

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
    // Bit data that represents u8, u16, u32, and u64 representations of incoming digits, converted
    // to a primitive upon parsing. It's also used to store content length.
    bit_data: u64,

    // Total byte count processed for headers, and body.
    // Once the headers are finished processing, this is reset to 0 to track the body length.
    byte_count: usize,

    // Content type.
    content_type: ContentType,

    // State details.
    flags: Flag,

    // Current state.
    state: State,

    // Current state function.
    state_function: StateFunction<T>
}

impl<T: HttpHandler + ParamHandler> Parser<T> {
    /// Create a new `Parser`.
    pub fn new(state: State, state_function: StateFunction<T>) -> Parser<T> {
        Parser{ bit_data:       0,
                byte_count:     0,
                content_type:   ContentType::None,
                flags:          F_NONE,
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
    /// If `Ok()` is returned, one of two things have happened: parsing has finished, or
    /// a callback function has preemptively stopped parsing. In either of these events, it
    /// is ok to call `parse()` again with a new slice of data to continue where the parser
    /// left off. `ParserError::Eof` is the only `ParserError` that allows `parse()` to
    /// continue parsing. Any other `ParserError` is a protocol error.
    ///
    /// Returns the parsed byte count of the current slice when parsing completes, or when
    /// a callback function preemptively stops parsing. Otherwise `ParserError`.
    #[inline]
    pub fn parse(&mut self, handler: &mut T, mut stream: &[u8]) -> Result<Success, ParserError> {
        let mut context = ParserContext::new(handler, stream);

        loop {
            match (self.state_function)(self, &mut context) {
                Ok(ParserValue::Continue) => {
                },
                Ok(ParserValue::ShiftStream(length)) => {
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
            if context.byte == b'L' {
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
            } else if context.byte == b'C' {
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
                if self.flags.contains(F_CHUNK_DATA) {
                    exit_finished!(self, context);
                }

                change_state_fast!(self, context);
            } else if self.flags.contains(F_CHUNK_DATA) {
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
            self.bit_data = 0;

            set_state!(self, State::RequestVersionMajor, request_version_major);
            change_state_fast!(self, context);
        } else {
            exit_error!(self, context, ParserError::Version(ERR_VERSION, context.byte));
        }
    }

    #[inline]
    pub fn request_version_major(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = self.bit_data as u16;

        collect_digits!(self, context, digit, 999, ParserError::Version, ERR_VERSION, {
            self.bit_data = digit as u64;

            exit_eof!(self, context);
        });

        self.bit_data = (digit as u64) << 4;

        if context.byte != b'.' {
            exit_error!(self, context, ParserError::Version(ERR_VERSION, context.byte));
        }

        set_state!(self, State::RequestVersionMinor, request_version_minor);
        change_state_fast!(self, context);
    }

    #[inline]
    pub fn request_version_minor(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit: u16 = (self.bit_data & 0xF) as u16;

        collect_digits!(self, context, digit, 999, ParserError::Version, ERR_VERSION, {
            self.bit_data += digit as u64;

            exit_eof!(self, context);
        });

        set_state!(self, State::PreHeaders1, pre_headers1);

        if context.handler.on_version((self.bit_data >> 4) as u16, digit) {
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
            self.bit_data = 0;

            set_state!(self, State::ResponseVersionMajor, response_version_major);
            change_state_fast!(self, context);
        } else {
            exit_error!(self, context, ParserError::Version(ERR_VERSION, context.byte));
        }
    }

    #[inline]
    pub fn response_version_major(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = self.bit_data as u16;

        collect_digits!(self, context, digit, 999, ParserError::Version, ERR_VERSION, {
            self.bit_data = digit as u64;

            exit_eof!(self, context);
        });

        self.bit_data = (digit as u64) << 4;

        if context.byte != b'.' {
            exit_error!(self, context, ParserError::Version(ERR_VERSION, context.byte));
        }

        set_state!(self, State::ResponseVersionMinor, response_version_minor);
        change_state_fast!(self, context);
    }

    #[inline]
    pub fn response_version_minor(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit: u16 = (self.bit_data & 0xF) as u16;

        collect_digits!(self, context, digit, 999, ParserError::Version, ERR_VERSION, {
            self.bit_data += digit as u64;

            exit_eof!(self, context);
        });

        set_state!(self, State::StripResponseStatusCode, strip_response_status_code);

        if context.handler.on_version((self.bit_data >> 4) as u16, digit) {
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

        self.bit_data = 0;

        set_state!(self, State::ResponseStatusCode, response_status_code);
        change_state_fast!(self, context);
    }

    #[inline]
    pub fn response_status_code(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = self.bit_data as u16;

        collect_digits!(self, context, digit, 999, ParserError::StatusCode, ERR_STATUS_CODE, {
            self.bit_data = digit as u64;

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
        exit_eof!(self, context);
    }

    #[inline]
    pub fn content(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn chunk_size(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn chunk_extension(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn chunk_size_newline1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    pub fn chunk_size_newline2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
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
