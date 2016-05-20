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
        $handler.set_content_type(ContentType::UrlEncoded);

        setup(&mut $parser, &mut $handler, b"GET / HTTP/1.1\r\n\r\n", State::UrlEncodedField);
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_visible(b"\r", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        setup!(p, h);

        if let ParserError::UrlEncodedField(x) = assert_error(&mut p, &mut h, &[byte]).unwrap() {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid bytes
    loop_visible(b"=%&", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        setup!(p, h);

        assert_eof(&mut p, &mut h, &[byte], State::UrlEncodedField, 1);
    });
}

#[test]
fn callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn get_content_type(&mut self) -> ContentType {
            ContentType::UrlEncoded
        }

        fn on_url_encoded_field(&mut self, _field: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new_request();

    setup(&mut p, &mut h, b"GET / HTTP/1.1\r\n\r\n", State::UrlEncodedField);

    assert_callback(&mut p, &mut h, b"Field", State::UrlEncodedField, 5);
}

#[test]
fn field() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"Field", State::UrlEncodedField, 5);
    assert_eq!(h.url_encoded_field, b"Field");
}

#[test]
fn field_ending_ampersand() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"Field&", State::UrlEncodedField, 6);
    assert_eq!(h.url_encoded_field, b"Field");
}

#[test]
fn field_ending_equal() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"Field=", State::UrlEncodedValue, 6);
    assert_eq!(h.url_encoded_field, b"Field");
}

#[test]
fn field_ending_percent() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"Field%", State::UrlEncodedFieldHex, 6);
    assert_eq!(h.url_encoded_field, b"Field");
}

#[test]
fn field_ending_plus() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"Field+", State::UrlEncodedField, 6);
    assert_eq!(h.url_encoded_field, b"Field ");
}

#[test]
fn field_hex() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"Field%21", State::UrlEncodedField, 8);
    assert_eq!(h.url_encoded_field, b"Field!");
}

#[test]
fn hex_error() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    if let ParserError::UrlEncodedField(x) = assert_error(&mut p, &mut h, b"%2z").unwrap() {
        assert_eq!(x, b'%');
    } else {
        panic!();
    }
}
