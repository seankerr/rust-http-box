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
    ($parser:expr, $handler:expr, $length:expr) => ({
        $parser.init_url_encoded($length);
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_visible(b"\r", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h, 1000);

        assert_error_byte!(p, h,
                           &[byte],
                           ParserError::UrlEncodedName,
                           byte);
    });

    // valid bytes
    loop_visible(b"=%&", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h, 1);

        assert_finished!(p, h,
                         &[byte]);
    });
}

#[test]
fn callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn on_url_encoded_name(&mut self, _field: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new();

    setup!(p, h, 1000);

    assert_callback!(p, h,
                     b"Field",
                     ParserState::UrlEncodedName);
}

#[test]
fn basic() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h, 1000);

    assert_eos!(p, h,
                b"Field",
                ParserState::UrlEncodedName);
    assert_eq!(h.url_encoded_name, b"Field");
}

#[test]
fn ending_ampersand() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h, 1000);

    assert_eos!(p, h,
                b"Field1&",
                ParserState::UrlEncodedName);
    assert_eq!(h.url_encoded_name, b"Field1");
}

#[test]
fn ending_equal() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h, 1000);

    assert_eos!(p, h,
                b"Field=",
                ParserState::UrlEncodedValue);
    assert_eq!(h.url_encoded_name, b"Field");
}

#[test]
fn ending_percent() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h, 1000);

    assert_eos!(p, h,
                b"Field%",
                ParserState::UrlEncodedNameHex1);
    assert_eq!(h.url_encoded_name, b"Field");
}

#[test]
fn ending_plus() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h, 1000);

    assert_eos!(p, h,
                b"Field+",
                ParserState::UrlEncodedName);
    assert_eq!(h.url_encoded_name, b"Field ");
}

#[test]
fn hex() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h, 1000);

    assert_eos!(p, h,
                b"Field%21",
                ParserState::UrlEncodedName);
    assert_eq!(h.url_encoded_name, b"Field!");
}

#[test]
fn hex_error() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h, 1000);

    assert_error_byte!(p, h,
                       b"%2z",
                       ParserError::UrlEncodedName,
                       b'z');
}
