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
        parser
    });

    () => ({
        let mut parser = Parser::new_url_encoded(DebugHandler::new());

        parser.set_length(1000);
        parser
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_visible(b"\r", |byte| {
        let mut p = setup!();

        assert_error_byte!(p,
                           &[byte],
                           UrlEncodedName,
                           byte);
    });

    // valid bytes
    loop_visible(b"=%&", |byte| {
        let mut p = setup!(1);

        assert_finished!(p,
                         &[byte]);
    });
}

#[test]
fn callback_exit() {
    struct CallbackHandler;

    impl HttpHandler for CallbackHandler {
        fn on_url_encoded_name(&mut self, _field: &[u8]) -> bool {
            false
        }
    }

    let mut p = Parser::new_url_encoded(CallbackHandler);

    p.set_length(1000);

    assert_callback!(p,
                     b"Field",
                     UrlEncodedName);
}

#[test]
fn basic() {
    let mut p = setup!();

    assert_eos!(p,
                b"Field",
                UrlEncodedName);

    assert_eq!(p.handler().url_encoded_name,
               b"Field");
}

#[test]
fn ending_ampersand() {
    let mut p = setup!();

    assert_eos!(p,
                b"Field1&",
                UrlEncodedName);

    assert_eq!(p.handler().url_encoded_name,
               b"Field1");
}

#[test]
fn ending_equal() {
    let mut p = setup!();

    assert_eos!(p,
                b"Field=",
                UrlEncodedValue);

    assert_eq!(p.handler().url_encoded_name,
               b"Field");
}

#[test]
fn ending_percent() {
    let mut p = setup!();

    assert_eos!(p,
                b"Field%",
                UrlEncodedNameHex1);

    assert_eq!(p.handler().url_encoded_name,
               b"Field");
}

#[test]
fn ending_plus() {
    let mut p = setup!();

    assert_eos!(p,
                b"Field+",
                UrlEncodedName);

    assert_eq!(p.handler().url_encoded_name,
               b"Field ");
}

#[test]
fn hex() {
    let mut p = setup!();

    assert_eos!(p,
                b"Field%21",
                UrlEncodedName);

    assert_eq!(p.handler().url_encoded_name,
               b"Field!");
}

#[test]
fn hex_error() {
    let mut p = setup!();

    assert_error_byte!(p,
                       b"%2z",
                       UrlEncodedName,
                       b'z');
}
