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

use handler::debug::*;
use http1::*;
use test::*;
use test::http1::*;

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        chunked_setup(&mut $parser, &mut $handler, b"F;extension=",
                      ParserState::ChunkExtensionValue);
    });
}

#[test]
fn byte_check_unquoted() {
    // invalid bytes
    loop_non_tokens(b"\r;=\"", |byte| {
        let mut h = DebugHttp1Handler::new();
        let mut p = Parser::new();

        setup!(p, h);

        if let ParserError::ChunkExtensionValue(x) = chunked_assert_error(&mut p, &mut h,
                                                                          &[byte]).unwrap() {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid bytes
    loop_tokens(b"", |byte| {
        let mut h = DebugHttp1Handler::new();
        let mut p = Parser::new();

        setup!(p, h);

        chunked_assert_eos(&mut p, &mut h, &[byte], ParserState::ChunkExtensionValue, 1);
    });
}

#[test]
fn basic() {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    chunked_assert_eos(&mut p, &mut h, b"valid-value;", ParserState::ChunkExtensionName, 12);
    assert_eq!(h.chunk_extension_value, b"valid-value");
}

#[test]
fn callback_exit() {
    struct X;

    impl Http1Handler for X {
        fn on_chunk_extension_value(&mut self, _value: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new();

    setup!(p, h);

    assert_callback(&mut p, &mut h, b"ExtensionValue", ParserState::ChunkExtensionValue, 14);
}

#[test]
fn repeat() {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    chunked_assert_eos(&mut p, &mut h, b"valid-value\r", ParserState::ChunkLengthNewline, 12);
    assert_eq!(h.chunk_extension_value, b"valid-value");
}
