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
    ($parser:expr, $handler:expr, $data_length:expr) => ({
        url_encoded_setup(&mut $parser, &mut $handler, b"Field=", State::UrlEncodedValue,
                          $data_length);
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_visible(b"\r", |byte| {
        let mut h = DebugHttp1Handler::new();
        let mut p = Parser::new();

        setup!(p, h, 7);

        if let ParserError::UrlEncodedValue(x) = url_encoded_assert_error(&mut p, &mut h,
                                                                          &[byte], 1).unwrap() {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid bytes
    loop_visible(b"&%=", |byte| {
        let mut h = DebugHttp1Handler::new();
        let mut p = Parser::new();

        setup!(p, h, 7);

        url_encoded_assert_eos(&mut p, &mut h, &[byte], State::UrlEncodedValue, 1, 1);
    });
}

#[test]
fn callback_exit() {
    struct X;

    impl Http1Handler for X {
        fn on_url_encoded_value(&mut self, _value: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new();

    setup!(p, h, 11);

    url_encoded_assert_callback(&mut p, &mut h, b"Value", State::UrlEncodedValue, 11, 5);
}

#[test]
fn equal_error() {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h, 7);

    if let ParserError::UrlEncodedValue(x) = url_encoded_assert_error(&mut p, &mut h,
                                                                      b"=", 7).unwrap() {
        assert_eq!(x, b'=');
    } else {
        panic!();
    }
}

#[test]
fn hex_error() {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h, 9);

    if let ParserError::UrlEncodedValue(x) = url_encoded_assert_error(&mut p, &mut h,
                                                                      b"%2z", 9).unwrap() {
        assert_eq!(x, b'%');
    } else {
        panic!();
    }
}

#[test]
fn value() {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h, 11);

    url_encoded_assert_eos(&mut p, &mut h, b"Value", State::UrlEncodedValue, 11, 5);
    assert_eq!(h.url_encoded_value, b"Value");
}

#[test]
fn value_ending_ampersand() {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h, 12);

    url_encoded_assert_eos(&mut p, &mut h, b"Value&", State::UrlEncodedField, 12, 6);
    assert_eq!(h.url_encoded_value, b"Value");
}

#[test]
fn value_ending_percent() {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h, 12);

    url_encoded_assert_eos(&mut p, &mut h, b"Value%", State::UrlEncodedValueHex, 12, 6);
    assert_eq!(h.url_encoded_value, b"Value");
}

#[test]
fn value_ending_plus() {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h, 12);

    url_encoded_assert_eos(&mut p, &mut h, b"Value+", State::UrlEncodedValue, 12, 6);
    assert_eq!(h.url_encoded_value, b"Value ");
}

#[test]
fn value_hex() {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h, 14);

    url_encoded_assert_eos(&mut p, &mut h, b"Value%21", State::UrlEncodedValue, 14, 8);
    assert_eq!(h.url_encoded_value, b"Value!");
}
