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

use fsm::Success;
use http1::{ HttpHandler, Parser, ParserError, ParserState };

use std::str;

macro_rules! http1_setup {
    () => (
        (Parser::new(), DebugHandler::new())
    )
}

fn assert_callback<T: HttpHandler>(
    parser:  &mut Parser<T>,
    handler: &mut T,
    stream:  &[u8],
    state:   ParserState,
    length:  usize
) {
    match parser.resume(handler, stream) {
        Ok(Success::Callback(length_)) => {
            assert_eq!(length, length_);
            assert_eq!(state, parser.state());
        },
        _ => panic!("assert_callback() Ok() match failed")
    }
}

fn assert_eos<T: HttpHandler>(
    parser:  &mut Parser<T>,
    handler: &mut T,
    stream:  &[u8],
    state:   ParserState,
    length:  usize
) {
    match parser.resume(handler, stream) {
        Ok(Success::Eos(length_)) => {
            assert_eq!(length, length_);
            assert_eq!(state, parser.state());
        },
        _ => panic!("assert_eos() Ok() match failed")
    }
}

fn assert_error<T: HttpHandler>(
    parser:  &mut Parser<T>,
    handler: &mut T,
    stream:  &[u8],
    error:   ParserError
) {
    match parser.resume(handler, stream) {
        Err(error_) => {
            assert_eq!(error, error_);
            assert_eq!(ParserState::Dead, parser.state());
        },
        _ => panic!("assert_error() Err() match failed")
    }
}

fn iter_assert_eos<T: HttpHandler>(
    parser:  &mut Parser<T>,
    handler: &mut T,
    details: &[(u8, ParserState)]
) {
    for &(byte, state) in details.iter() {
        match parser.resume(handler, &[byte]) {
            Ok(Success::Eos(length)) => {
                println!(
                    "iter_assert_eos() state: expected {:?} is {:?}",
                    state,
                    parser.state()
                );
                assert_eq!(length, 1);
                //assert_eq!(item.1[n], parser.state());
            },
            _ => panic!("iter_assert_eos() Success::Eos match failed")
        }
    }
}

fn assert_finished<T: HttpHandler>(
    parser:  &mut Parser<T>,
    handler: &mut T,
    stream:  &[u8],
    length:  usize
) {
    match parser.resume(handler, stream) {
        Ok(Success::Finished(length_)) => {
            assert_eq!(length, length_);
            assert_eq!(ParserState::Finished, parser.state());
        },
        _ => panic!("assert_finished() Ok() match failed")
    }
}

/// `DebugHandler` works with all `http1::Parser` parsing methods.
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

    /// Indicates that the most recent chunk extension has been parsed.
    pub chunk_extension_finished: bool,

    /// Chunk extension name.
    pub chunk_extension_name:  Vec<u8>,

    /// Chunk extension value.
    pub chunk_extension_value: Vec<u8>,

    /// Indicates that all chunk extensions have been parsed.
    pub chunk_extensions_finished: bool,

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
        DebugHandler{
            body_finished:             false,
            chunk_data:                Vec::new(),
            chunk_extension_finished:  false,
            chunk_extension_name:      Vec::new(),
            chunk_extension_value:     Vec::new(),
            chunk_extensions_finished: false,
            chunk_length:              0,
            header_name:               Vec::new(),
            header_value:              Vec::new(),
            headers_finished:          false,
            initial_finished:          false,
            method:                    Vec::new(),
            multipart_data:            Vec::new(),
            status:                    Vec::new(),
            status_code:               0,
            url:                       Vec::new(),
            url_encoded_name:          Vec::new(),
            url_encoded_value:         Vec::new(),
            version_major:             0,
            version_minor:             0
        }
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
            if is_not_visible_7bit!(*byte) && *byte != b' ' {
                println!("on_chunk_data [{}]: *hidden*", data.len());
                return true;
            }
        }

        println!("on_chunk_data [{}]: {:?}", data.len(), str::from_utf8(data).unwrap());
        true
    }

    fn on_chunk_extension_finished(&mut self) -> bool {
        self.chunk_extension_finished = true;
        true
    }

    fn on_chunk_extension_name(&mut self, name: &[u8]) -> bool {
        self.chunk_extension_name.extend_from_slice(name);
        println!("on_chunk_extension_name [{}]: {:?}", name.len(), str::from_utf8(name).unwrap());
        true
    }

    fn on_chunk_extension_value(&mut self, value: &[u8]) -> bool {
        self.chunk_extension_value.extend_from_slice(value);

        for byte in value {
            if is_not_visible_7bit!(*byte) && *byte != b' ' {
                println!("on_chunk_extension_value [{}]: *hidden*", value.len());
                return true;
            }
        }

        true
    }

    fn on_chunk_extensions_finished(&mut self) -> bool {
        self.chunk_extensions_finished = true;
        true
    }

    fn on_chunk_length(&mut self, length: usize) -> bool {
        self.chunk_length = length;
        println!("on_chunk_length: {}", length);
        true
    }

    fn on_header_name(&mut self, name: &[u8]) -> bool {
        self.header_name.extend_from_slice(name);
        println!("on_header_name [{}]: {:?}", name.len(), str::from_utf8(name).unwrap());
        true
    }

    fn on_header_value(&mut self, value: &[u8]) -> bool {
        self.header_value.extend_from_slice(value);

        for byte in value {
            if is_not_visible_7bit!(*byte) && *byte != b' ' {
                println!("on_header_value [{}]: *hidden*", value.len());
                return true;
            }
        }

        println!("on_header_value [{}]: {:?}", value.len(), str::from_utf8(value).unwrap());
        true
    }

    fn on_headers_finished(&mut self) -> bool {
        self.headers_finished = true;
        println!("on_headers_finished");
        true
    }

    fn on_initial_finished(&mut self) -> bool {
        self.initial_finished = true;
        println!("on_initial_finished");
        true
    }

    fn on_method(&mut self, method: &[u8]) -> bool {
        self.method.extend_from_slice(method);
        println!("on_method [{}]: {:?}", method.len(), str::from_utf8(method).unwrap());
        true
    }

    fn on_multipart_begin(&mut self) -> bool {
        println!("on_multipart_begin");
        true
    }

    fn on_multipart_data(&mut self, data: &[u8]) -> bool {
        self.multipart_data.extend_from_slice(data);

        for byte in data {
            if is_not_visible_7bit!(*byte) {
                println!("on_multipart_data [{}]: *hidden*", data.len());
                return true;
            }
        }

        println!("on_multipart_data [{}]: {:?}", data.len(), str::from_utf8(data).unwrap());
        true
    }

    fn on_status(&mut self, status: &[u8]) -> bool {
        self.status.extend_from_slice(status);

        for byte in status {
            if is_not_visible_7bit!(*byte) {
                println!("on_status [{}]: *hidden*", status.len());
                return true;
            }
        }

        true
    }

    fn on_status_code(&mut self, code: u16) -> bool {
        self.status_code = code;
        println!("on_status_code: {}", code);
        true
    }

    fn on_url(&mut self, url: &[u8]) -> bool {
        self.url.extend_from_slice(url);
        println!("on_url [{}]: {:?}", url.len(), str::from_utf8(url).unwrap());
        true
    }

    fn on_url_encoded_begin(&mut self) -> bool {
        println!("on_url_encoded_begin");
        true
    }

    fn on_url_encoded_name(&mut self, name: &[u8]) -> bool {
        self.url_encoded_name.extend_from_slice(name);
        println!("on_url_encoded_name [{}]: {:?}", name.len(), str::from_utf8(name).unwrap());
        true
    }

    fn on_url_encoded_value(&mut self, value: &[u8]) -> bool {
        self.url_encoded_value.extend_from_slice(value);
        println!("on_url_encoded_value [{}]: {:?}", value.len(), str::from_utf8(value).unwrap());
        true
    }

    fn on_version(&mut self, major: u16, minor: u16) -> bool {
        self.version_major = major;
        self.version_minor = minor;
        println!("on_version: {}.{}", major, minor);
        true
    }
}

mod chunked;
mod head;
mod multipart;
mod request;
mod response;
mod url_encoded;
