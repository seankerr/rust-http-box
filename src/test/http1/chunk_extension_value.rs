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

use handler::*;
use http1::*;
use test::*;
use test::http1::*;

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        $handler.set_transfer_encoding(TransferEncoding::Chunked);

        setup(&mut $parser, &mut $handler, b"GET / HTTP/1.1\r\n\r\nF;extension=",
              State::ChunkExtensionValue);
    });
}

#[test]
fn byte_check_unquoted() {
    // invalid bytes
    loop_non_tokens(b"\r;=\"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        setup!(p, h);

        if let ParserError::ChunkExtensionValue(x) = assert_error(&mut p, &mut h,
                                                                  &[byte]).unwrap() {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid bytes
    loop_tokens(b"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        setup!(p, h);

        assert_eos(&mut p, &mut h, &[byte], State::ChunkExtensionValue, 1);
    });
}

#[test]
fn basic() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"valid-value;", State::ChunkExtensionName, 12);
    assert_eq!(h.chunk_extension_value, b"valid-value");
}

#[test]
fn callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn get_transfer_encoding(&mut self) -> TransferEncoding {
            TransferEncoding::Chunked
        }

        fn on_chunk_extension_value(&mut self, _value: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new_request();

    setup(&mut p, &mut h, b"GET / HTTP/1.1\r\n\r\nF;", State::ChunkExtensionName);

    assert_callback(&mut p, &mut h, b"ChunkExtension=ExtensionValue", State::ChunkExtensionValue, 29);
}

#[test]
fn repeat() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"valid-value\r", State::ChunkSizeNewline, 12);
    assert_eq!(h.chunk_extension_value, b"valid-value");
}
