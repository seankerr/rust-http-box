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
// | Author: Sean Kerr <sean@metatomic.io>                                                         |
// +-----------------------------------------------------------------------------------------------+

use http1::*;
use test::*;
use test::http1::*;

macro_rules! setup {
    ($length:expr) => ({
        let mut handler = DebugHandler::new();
        let mut parser  = Parser::new();

        parser.init_url_encoded();
        parser.set_length($length);
        (parser, handler)
    });

    () => ({
        let mut handler = DebugHandler::new();
        let mut parser  = Parser::new();

        parser.init_url_encoded();
        parser.set_length(1000);
        (parser, handler)
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_visible(
        b"\r",
        |byte| {
            let (mut p, mut h) = setup!();

            assert_error_byte!(
                p,
                h,
                &[byte],
                UrlEncodedName,
                byte
            );
        }
    );

    // valid bytes
    loop_visible(
        b"=%&",
        |byte| {
            let (mut p, mut h) = setup!(1);

            assert_finished!(
                p,
                h,
                &[byte]
            );
        }
    );
}

#[test]
fn callback_exit() {
    struct CallbackHandler;

    impl HttpHandler for CallbackHandler {
        fn on_url_encoded_name(&mut self, _field: &[u8]) -> bool {
            false
        }
    }

    let mut h = CallbackHandler;
    let mut p = Parser::new();

    p.init_url_encoded();
    p.set_length(1000);

    assert_callback!(
        p,
        h,
        b"Field",
        UrlEncodedName
    );
}

#[test]
fn basic() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Field",
        UrlEncodedName
    );

    assert_eq!(
        h.url_encoded_name,
        b"Field"
    );
}

#[test]
fn ending_ampersand() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Field1&",
        UrlEncodedName
    );

    assert_eq!(
        h.url_encoded_name,
        b"Field1"
    );
}

#[test]
fn ending_equal() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Field=",
        UrlEncodedValue
    );

    assert_eq!(
        h.url_encoded_name,
        b"Field"
    );
}

#[test]
fn ending_percent() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Field%",
        UrlEncodedNameHex1
    );

    assert_eq!(
        h.url_encoded_name,
        b"Field"
    );
}

#[test]
fn ending_plus() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Field+",
        UrlEncodedName
    );

    assert_eq!(
        h.url_encoded_name,
        b"Field "
    );
}

#[test]
fn hex() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Field%21",
        UrlEncodedName
    );

    assert_eq!(
        h.url_encoded_name,
        b"Field!"
    );
}

#[test]
fn hex_error() {
    let (mut p, mut h) = setup!();

    assert_error_byte!(
        p,
        h,
        b"%2z",
        UrlEncodedName,
        b'z'
    );
}
