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
        setup(&mut $parser, &mut $handler, b"HTTP/1.0 ", ParserState::StripResponseStatusCode);
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_digits(b" \t", |byte| {
        let mut h = DebugHttp1Handler::new();
        let mut p = Parser::new();

        setup!(p, h);

        if let ParserError::StatusCode(x) = assert_error(&mut p, &mut h, &[byte]).unwrap() {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid bytes
    loop_digits(b"", |byte| {
        let mut h = DebugHttp1Handler::new();
        let mut p = Parser::new();

        setup!(p, h);

        assert_eos(&mut p, &mut h, &[byte], ParserState::ResponseStatusCode, 1);
    });
}

#[test]
fn callback_exit() {
    struct X;

    impl Http1Handler for X {
        fn on_status_code(&mut self, _code: u16) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new();

    setup!(p, h);

    assert_callback(&mut p, &mut h, b"100 ", ParserState::StripResponseStatus, 3);
}

#[test]
fn v0 () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"0 ", ParserState::StripResponseStatus, 2);
    assert_eq!(h.status_code, 0);
}

#[test]
fn v999 () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"999 ", ParserState::StripResponseStatus, 4);
    assert_eq!(h.status_code, 999);
}

#[test]
fn v1000 () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    if let ParserError::StatusCode(x) = assert_error(&mut p, &mut h, b"1000").unwrap() {
        assert_eq!(x, b'0');
    } else {
        panic!();
    }
}
