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
        setup(&mut $parser, &mut $handler, b"GET / HTTP/1.1\r\nFieldName: ", ParserState::StripHeaderValue);
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_visible(b"\r\t ", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        if let ParserError::HeaderValue(x) = assert_error(&mut p, &mut h, &[byte]).unwrap() {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid bytes
    loop_visible(b"\"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        assert_eos(&mut p, &mut h, &[byte], ParserState::HeaderValue, 1);
    });
}

#[test]
fn callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn on_header_value(&mut self, _field: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new();

    setup!(p, h);

    assert_callback(&mut p, &mut h, b"F", ParserState::HeaderValue, 1);
}

#[test]
fn multiline() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Value1\r\n", ParserState::HeaderNewline3, 8);
    assert_eq!(h.header_value, b"Value1");
    assert_eos(&mut p, &mut h, b" Value2\r", ParserState::HeaderNewline2, 8);
    assert_eq!(h.header_value, b"Value1 Value2");
}

#[test]
fn multiple() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Value", ParserState::HeaderValue, 5);
    assert_eq!(h.header_value, b"Value");
    assert_eos(&mut p, &mut h, b"Time\r", ParserState::HeaderNewline2, 5);
    assert_eq!(h.header_value, b"ValueTime");
}

#[test]
fn single() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"ValueTime\r", ParserState::HeaderNewline2, 10);
    assert_eq!(h.header_value, b"ValueTime");
}

#[test]
fn space() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Value Time\r", ParserState::HeaderNewline2, 11);
    assert_eq!(h.header_value, b"Value Time");
}
