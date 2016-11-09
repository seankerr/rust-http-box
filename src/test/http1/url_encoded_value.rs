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

        assert_eos!($parser, $handler,
                    b"Field=",
                    ParserState::UrlEncodedValue);
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
                           ParserError::UrlEncodedValue,
                           byte);
    });

    // valid bytes
    loop_visible(b"&%=", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h, 7);

        assert_finished!(p, h,
                         &[byte]);
    });
}

#[test]
fn callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn on_url_encoded_value(&mut self, _value: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new();

    setup!(p, h, 1000);

    assert_callback!(p, h,
                     b"Value",
                     ParserState::UrlEncodedValue);
}

#[test]
fn equal_error() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h, 7);

    assert_error_byte!(p, h,
                       b"=",
                       ParserError::UrlEncodedValue,
                       b'=');
}

#[test]
fn full_complex() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h, 37);

    assert_finished!(p, h,
                     b"Value&Field%202%21=Value%202%21");
}

#[test]
fn full_simple() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h, 25);

    assert_finished!(p, h,
                     b"Value&Field2=Value2");
}

#[test]
fn hex_error() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h, 1000);

    assert_error_byte!(p, h,
                       b"%2z",
                       ParserError::UrlEncodedValue,
                       b'z');
}

#[test]
fn value() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h, 1000);

    assert_eos!(p, h,
                b"Value",
                ParserState::UrlEncodedValue);
    assert_eq!(h.url_encoded_value, b"Value");
}

#[test]
fn value_ending_ampersand() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h, 1000);

    assert_eos!(p, h,
                b"Value&",
                ParserState::UrlEncodedName);
    assert_eq!(h.url_encoded_value, b"Value");
}

#[test]
fn value_ending_percent() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h, 1000);

    assert_eos!(p, h,
               b"Value%",
               ParserState::UrlEncodedValueHex1);
    assert_eq!(h.url_encoded_value, b"Value");
}

#[test]
fn value_ending_plus() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h, 1000);

    assert_eos!(p, h,
                b"Value+",
                ParserState::UrlEncodedValue);
    assert_eq!(h.url_encoded_value, b"Value ");
}

#[test]
fn value_hex() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h, 1000);

    assert_eos!(p, h,
                b"Value%21",
                ParserState::UrlEncodedValue);
    assert_eq!(h.url_encoded_value, b"Value!");
}
