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
        setup(&mut $parser, &mut $handler, b"HTTP/1.1 200 ", ParserState::StripResponseStatus);
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_tokens(b"\r\t ", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        if let ParserError::Status(x) = assert_error(&mut p, &mut h, &[byte]).unwrap() {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid bytes
    loop_tokens(b"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        assert_eos(&mut p, &mut h, &[byte], ParserState::ResponseStatus, 1);
    });
}

#[test]
fn callback_exit() {
    struct X;

    impl Http1Handler for X {
        fn on_status(&mut self, _status: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new();

    setup!(p, h);

    assert_callback(&mut p, &mut h, b"A\tCOOL STATUS\r", ParserState::StatusEnd, 14);
}

#[test]
fn multiple() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"NOT ", ParserState::ResponseStatus, 4);
    assert_eq!(h.status, b"NOT ");
    assert_eos(&mut p, &mut h, b"FOUND\r", ParserState::PreHeaders1, 6);
    assert_eq!(h.status, b"NOT FOUND");
}

#[test]
fn single() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"NOT FOUND\r", ParserState::PreHeaders1, 10);
    assert_eq!(h.status, b"NOT FOUND");
}
