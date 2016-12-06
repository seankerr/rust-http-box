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
// | Author: Sean Kerr <sean@metatomic.io>                                                         |
// +-----------------------------------------------------------------------------------------------+

use http1::*;
pub use fsm::Success;

use std::str;

macro_rules! assert_callback {
    ($parser:expr, $stream:expr, $state:ident, $length:expr) => ({
        assert!(match $parser.resume($stream) {
            Ok(Success::Callback(byte_count)) => {
                assert_eq!(byte_count, $length);
                assert_eq!($parser.state(), ParserState::$state);
                true
            },
            _ => false
        });
    });

    ($parser:expr, $stream:expr, $state:ident) => ({
        assert_callback!($parser, $stream, $state, $stream.len())
    });
}

macro_rules! assert_eos {
    ($parser:expr, $stream:expr, $state:ident, $length:expr) => ({
        assert!(match $parser.resume($stream) {
            Ok(Success::Eos(byte_count)) => {
                assert_eq!(byte_count, $length);
                assert_eq!($parser.state(), ParserState::$state);
                true
            },
            _ => false
        });
    });

    ($parser:expr, $stream:expr, $state:ident) => ({
        assert_eos!($parser, $stream, $state, $stream.len())
    });
}

macro_rules! assert_error {
    ($parser:expr, $stream:expr, $error:ident) => ({
        assert!(match $parser.resume($stream) {
            Err(error) => {
                assert_eq!(error, ParserError::$error);
                assert_eq!($parser.state(), ParserState::Dead);
                true
            },
            _ => {
                false
            }
        });
    });
}

macro_rules! assert_error_byte {
    ($parser:expr, $stream:expr, $error:ident, $byte:expr) => ({
        assert!(match $parser.resume($stream) {
            Err(ParserError::$error(byte)) => {
                assert_eq!(byte, $byte);
                assert_eq!($parser.state(), ParserState::Dead);
                true
            },
            _ => {
                false
            }
        });
    });
}

macro_rules! assert_finished {
    ($parser:expr, $stream:expr, $length:expr) => ({
        assert!(match $parser.resume($stream) {
            Ok(Success::Finished(byte_count)) => {
                assert_eq!(byte_count, $length);
            true
            },
            _ => false
        });
    });

    ($parser:expr, $stream:expr) => ({
        assert_finished!($parser, $stream, $stream.len())
    });
}

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

    /// Header name.
    pub header_name: Vec<u8>,

    /// Header value.
    pub header_value: Vec<u8>,

    /// Indicates that headers have successfully been parsed.
    pub headers_finished: bool,

    /// Indicates that the initial request/response line has successfully been parsed.
    pub initial_finished: bool,

    /// Request method.
    pub method: Vec<u8>,

    /// Multipart data.
    pub multipart_data: Vec<u8>,

    /// Response status.
    pub status: Vec<u8>,

    /// Response status code.
    pub status_code: u16,

    /// Request URL.
    pub url: Vec<u8>,

    /// URL encoded name.
    pub url_encoded_name: Vec<u8>,

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
                      header_name:           Vec::new(),
                      header_value:          Vec::new(),
                      headers_finished:      false,
                      initial_finished:      false,
                      method:                Vec::new(),
                      multipart_data:        Vec::new(),
                      status:                Vec::new(),
                      status_code:           0,
                      url:                   Vec::new(),
                      url_encoded_name:      Vec::new(),
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
        self.header_name           = Vec::new();
        self.header_value          = Vec::new();
        self.headers_finished      = false;
        self.initial_finished      = false;
        self.method                = Vec::new();
        self.multipart_data        = Vec::new();
        self.status                = Vec::new();
        self.status_code           = 0;
        self.url                   = Vec::new();
        self.url_encoded_name      = Vec::new();
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

    fn on_chunk_begin(&mut self) -> bool {
        println!("on_chunk_begin");
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

    fn on_header_name(&mut self, name: &[u8]) -> bool {
        println!("on_header_name [{}]: {:?}", name.len(), str::from_utf8(name).unwrap());
        self.header_name.extend_from_slice(name);
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

    fn on_initial_finished(&mut self) -> bool {
        println!("on_initial_finished");
        self.initial_finished = true;
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

    fn on_url(&mut self, url: &[u8]) -> bool {
        println!("on_url [{}]: {:?}", url.len(), str::from_utf8(url).unwrap());
        self.url.extend_from_slice(url);
        true
    }

    fn on_url_encoded_name(&mut self, name: &[u8]) -> bool {
        println!("on_url_encoded_name [{}]: {:?}", name.len(), str::from_utf8(name).unwrap());
        self.url_encoded_name.extend_from_slice(name);
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
// test mods
mod chunk_data;
mod chunk_extension_finished;
mod chunk_extension_name;
mod chunk_extension_quoted_value;
mod chunk_extension_value;
mod chunk_extensions_finished;
mod chunk_length;
mod chunk_trailer;

mod header_name;
mod header_quoted_value;
mod header_value;
mod headers_finished;

mod initial_finished;

mod multipart_begin;
mod multipart_boundary;
mod multipart_data;
mod multipart_header;

mod request_method;
mod request_url;
mod request_http;
mod request_version;

mod response_http;
mod response_version;
mod response_status_code;
mod response_status;

mod url_encoded_name;
mod url_encoded_value;
