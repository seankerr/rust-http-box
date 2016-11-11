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
                    b"F;extension1=",
                    ParserState::StripChunkExtensionValue);
    });
}

#[test]
fn basic() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"\"valid-value\"",
                ParserState::ChunkExtensionQuotedValueFinished);
    assert_eq!(h.chunk_extension_value, b"valid-value");
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_quoted(b"\r;\"\\", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        assert_eos!(p, h,
                    &[b'"'],
                    ParserState::ChunkExtensionQuotedValue);

        assert_error_byte!(p, h,
                           &[byte],
                           ParserError::ChunkExtensionValue,
                           byte);
    });

    // valid bytes
    loop_quoted(b"\"\\", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        assert_eos!(p, h,
                    &[b'"'],
                    ParserState::ChunkExtensionQuotedValue);
        assert_eos!(p, h,
                    &[byte],
                    ParserState::ChunkExtensionQuotedValue);
    });
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
                     b"\"ExtensionValue\"",
                     ParserState::ChunkExtensionQuotedValueFinished);
}

#[test]
fn escaped() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"\"valid \\\"value\\\" here\"\r",
                ParserState::ChunkLengthLf);
    assert_eq!(h.chunk_extension_value, b"valid \"value\" here");
}

#[test]
fn repeat() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"valid-value1;extension2=valid-value2;",
                ParserState::StripChunkExtensionName);
    assert_eq!(h.chunk_extension_name, b"extension1extension2");
    assert_eq!(h.chunk_extension_value, b"valid-value1valid-value2");
}
