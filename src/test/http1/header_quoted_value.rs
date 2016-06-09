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
        setup(&mut $parser, &mut $handler, b"GET / HTTP/1.1\r\nFieldName: ", State::StripHeaderValue);
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_quoted(b"\r;\"\\", |byte| {
        let mut h = DebugHttp1Handler::new();
        let mut p = Parser::new();

        setup!(p, h);

        assert_eos(&mut p, &mut h, &[b'"'], State::HeaderQuotedValue, 1);

        if let ParserError::HeaderValue(x) = assert_error(&mut p, &mut h,
                                                          &[byte]).unwrap() {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid bytes
    loop_quoted(b"\"\\", |byte| {
        let mut h = DebugHttp1Handler::new();
        let mut p = Parser::new();

        setup!(p, h);

        assert_eos(&mut p, &mut h, &[b'"'], State::HeaderQuotedValue, 1);
        assert_eos(&mut p, &mut h, &[byte], State::HeaderQuotedValue, 1);
    });
}

#[test]
fn escaped_multiple() {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"\"Value", State::HeaderQuotedValue, 6);
    assert_eq!(h.header_value, b"Value");
    assert_eos(&mut p, &mut h, b"\\\"", State::HeaderQuotedValue, 2);
    assert_eq!(h.header_value, b"Value\"");
    assert_eos(&mut p, &mut h, b"Time\"", State::Newline1, 5);
    assert_eq!(h.header_value, b"Value\"Time");
}

#[test]
fn escaped_single() {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"\"Value\\\"Time\"", State::Newline1, 13);
    assert_eq!(h.header_value, b"Value\"Time");
}

#[test]
fn multiple() {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"\"Value", State::HeaderQuotedValue, 6);
    assert_eq!(h.header_value, b"Value");
    assert_eos(&mut p, &mut h, b"Time\"", State::Newline1, 5);
    assert_eq!(h.header_value, b"ValueTime");
}

#[test]
fn single() {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"\"Value Time\"", State::Newline1, 12);
    assert_eq!(h.header_value, b"Value Time");
}
