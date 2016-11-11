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
use test::*;
use test::http1::*;

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        $parser.init_chunked();

        assert_eos!($parser, $handler,
                    b"F;extension=",
                    ParserState::StripChunkExtensionValue);
    });
}

#[test]
fn byte_check_unquoted() {
    // invalid bytes
    loop_non_tokens(b" \t\r;=\"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        assert_error_byte!(p, h,
                           &[byte],
                           ParserError::ChunkExtensionValue,
                           byte);
    });

    // valid bytes
    loop_tokens(b" \t", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        assert_eos!(p, h,
                    &[byte],
                    ParserState::ChunkExtensionValue);
    });
}

#[test]
fn basic() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"valid-value;",
                ParserState::StripChunkExtensionName);
    assert_eq!(h.chunk_extension_value, b"valid-value");
}

#[test]
fn callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn on_chunk_extension_value(&mut self, _value: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new();

    setup!(p, h);

    assert_callback!(p, h,
                     b"ExtensionValue",
                     ParserState::ChunkExtensionValue);
}

#[test]
fn linear_space() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"   \t\t\tvalid-value\r",
                ParserState::ChunkLengthLf);
    assert_eq!(h.chunk_extension_value, b"valid-value");
}

#[test]
fn repeat() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"valid-value\r",
                ParserState::ChunkLengthLf);
    assert_eq!(h.chunk_extension_value, b"valid-value");
}
