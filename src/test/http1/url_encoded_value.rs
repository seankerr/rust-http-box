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
    ($length:expr) => ({
        let mut parser = Parser::new_url_encoded(DebugHandler::new());

        parser.set_length($length);

        assert_eos!(parser,
                    b"Field=",
                    UrlEncodedValue);

        parser
    });

    () => ({
        setup!(1000)
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_visible(b"\r", |byte| {
        let mut p = setup!();

        assert_error_byte!(p,
                           &[byte],
                           UrlEncodedValue,
                           byte);
    });

    // valid bytes
    loop_visible(b"&%=", |byte| {
        let mut p = setup!(7);

        assert_finished!(p,
                         &[byte]);
    });
}

#[test]
fn callback_exit() {
    struct CallbackHandler;

    impl HttpHandler for CallbackHandler {
        fn on_url_encoded_value(&mut self, _value: &[u8]) -> bool {
            false
        }
    }

    let mut p = Parser::new_url_encoded(CallbackHandler);

    p.set_length(1000);

    assert_eos!(p,
                b"Field=",
                UrlEncodedValue);

    assert_callback!(p,
                     b"Value",
                     UrlEncodedValue);
}

#[test]
fn equal_error() {
    let mut p = setup!();

    assert_error_byte!(p,
                       b"=",
                       UrlEncodedValue,
                       b'=');
}

#[test]
fn full_complex() {
    let mut p = setup!(37);

    assert_finished!(p,
                     b"Value&Field%202%21=Value%202%21");
}

#[test]
fn full_simple() {
    let mut p = setup!(25);

    assert_finished!(p,
                     b"Value&Field2=Value2");
}

#[test]
fn hex_error() {
    let mut p = setup!();

    assert_error_byte!(p,
                       b"%2z",
                       UrlEncodedValue,
                       b'z');
}

#[test]
fn value() {
    let mut p = setup!();

    assert_eos!(p,
                b"Value",
                UrlEncodedValue);

    assert_eq!(p.handler().url_encoded_value,
               b"Value");
}

#[test]
fn value_ending_ampersand() {
    let mut p = setup!();

    assert_eos!(p,
                b"Value&",
                UrlEncodedName);

    assert_eq!(p.handler().url_encoded_value,
               b"Value");
}

#[test]
fn value_ending_percent() {
    let mut p = setup!();

    assert_eos!(p,
               b"Value%",
               UrlEncodedValueHex1);

    assert_eq!(p.handler().url_encoded_value,
               b"Value");
}

#[test]
fn value_ending_plus() {
    let mut p = setup!();

    assert_eos!(p,
                b"Value+",
                UrlEncodedValue);

    assert_eq!(p.handler().url_encoded_value,
               b"Value ");
}

#[test]
fn value_hex() {
    let mut p = setup!();

    assert_eos!(p,
                b"Value%21",
                UrlEncodedValue);

    assert_eq!(p.handler().url_encoded_value,
               b"Value!");
}
