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

use http1::*;
pub use fsm::Success;

macro_rules! assert_callback {
    ($parser:expr, $handler:expr, $stream:expr, $state:expr, $length:expr) => ({
        assert!(match $parser.resume(&mut $handler, $stream) {
            Ok(Success::Callback(byte_count)) => {
                assert_eq!(byte_count, $length);
                assert_eq!($parser.state(), $state);
                true
            },
            _ => false
        });
    });

    ($parser:expr, $handler:expr, $stream:expr, $state:expr) => ({
        assert_callback!($parser, $handler, $stream, $state, $stream.len())
    });
}

macro_rules! assert_eos {
    ($parser:expr, $handler:expr, $stream:expr, $state:expr, $length:expr) => ({
        assert!(match $parser.resume(&mut $handler, $stream) {
            Ok(Success::Eos(byte_count)) => {
                assert_eq!(byte_count, $length);
                assert_eq!($parser.state(), $state);
                true
            },
            _ => false
        });
    });

    ($parser:expr, $handler:expr, $stream:expr, $state:expr) => ({
        assert_eos!($parser, $handler, $stream, $state, $stream.len())
    });
}

macro_rules! assert_error {
    ($parser:expr, $handler:expr, $stream:expr, $error:expr) => ({
        assert!(match $parser.resume(&mut $handler, $stream) {
            Err(error) => {
                assert_eq!(error, $error);
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
    ($parser:expr, $handler:expr, $stream:expr, $error:path, $byte:expr) => ({
        assert!(match $parser.resume(&mut $handler, $stream) {
            Err($error(byte)) => {
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
    ($parser:expr, $handler:expr, $stream:expr, $length:expr) => ({
        assert!(match $parser.resume(&mut $handler, $stream) {
            Ok(Success::Finished(byte_count)) => {
                assert_eq!(byte_count, $length);
            true
            },
            _ => false
        });
    });

    ($parser:expr, $handler:expr, $stream:expr) => ({
        assert_finished!($parser, $handler, $stream, $stream.len())
    });
}

// test mods
mod chunk_data;
mod chunk_extension_finished;
mod chunk_extension_name;
mod chunk_extension_quoted_value;
mod chunk_extension_value;
mod chunk_extensions_finished;
mod chunk_size;
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
