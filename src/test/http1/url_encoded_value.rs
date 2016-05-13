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
use url::*;

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        $handler.set_content_type(ContentType::UrlEncoded);

        setup(&mut $parser, &mut $handler, b"GET / HTTP/1.1\r\n\r\nField=", State::UrlEncodedValue);
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_unsafe(b"\r", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        setup!(p, h);

        if let ParserError::UrlEncodedValue(x) = assert_error(&mut p, &mut h, &[byte]).unwrap() {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid bytes
    loop_safe(b"&%", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        setup!(p, h);

        assert_eof(&mut p, &mut h, &[byte], State::UrlEncodedValue, 1);
    });
}

#[test]
fn callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn get_content_type(&mut self) -> ContentType {
            ContentType::UrlEncoded
        }
    }

    impl ParamHandler for X {
        fn on_param_value(&mut self, _value: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new_request();

    setup(&mut p, &mut h, b"GET / HTTP/1.1\r\n\r\nValue=", State::UrlEncodedValue);

    assert_callback(&mut p, &mut h, b"Value", State::UrlEncodedValue, 5);
}

#[test]
fn finished() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_finished(&mut p, &mut h, b"Value\r\n", State::Finished, 7);
    assert_eq!(h.param_value, b"Value");
}

#[test]
fn hex_error() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    if let ParserError::HexSequence(x) = assert_error(&mut p, &mut h, b"%2z").unwrap() {
        assert_eq!(x, b'%');
    } else {
        panic!();
    }
}

#[test]
fn value() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"Value", State::UrlEncodedValue, 5);
    assert_eq!(h.param_value, b"Value");
}

#[test]
fn value_ending_ampersand() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"Value&", State::UrlEncodedField, 6);
    assert_eq!(h.param_value, b"Value");
}

#[test]
fn value_ending_percent() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"Value%", State::UrlEncodedValueHex, 6);
    assert_eq!(h.param_value, b"Value");
}

#[test]
fn value_ending_plus() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"Value+", State::UrlEncodedValue, 6);
    assert_eq!(h.param_value, b"Value ");
}

#[test]
fn value_hex() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"Value%21", State::UrlEncodedValue, 8);
    assert_eq!(h.param_value, b"Value!");
}
