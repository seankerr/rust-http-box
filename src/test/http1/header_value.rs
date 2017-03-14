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
    () => ({
        let mut handler = DebugHandler::new();
        let mut parser  = Parser::new();

        assert_eos!(
            parser,
            handler,
            b"GET / HTTP/1.1\r\nFieldName: ",
            StripHeaderValue
        );

        (parser, handler)
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_visible(
        b"\r\t ",
        |byte| {
            let (mut p, mut h) = setup!();

            assert_error_byte!(
                p,
                h,
                &[byte],
                HeaderValue,
                byte
            );
        }
    );

    // valid bytes
    loop_visible(
        b"\"",
        |byte| {
            let (mut p, mut h) = setup!();

            assert_eos!(
                p,
                h,
                &[byte],
                HeaderValue
            );
        }
    );
}

#[test]
fn callback_exit() {
    struct CallbackHandler;

    impl HttpHandler for CallbackHandler {
        fn on_header_value(&mut self, _field: &[u8]) -> bool {
            false
        }
    }

    let mut h = CallbackHandler;
    let mut p = Parser::new();

    assert_eos!(
        p,
        h,
        b"GET / HTTP/1.1\r\nFieldName: ",
        StripHeaderValue
    );

    assert_callback!(
        p,
        h,
        b"F",
        HeaderValue
    );
}

#[test]
fn multiline() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Value1\r\n",
        HeaderCr2
    );

    assert_eq!(
        h.header_value,
        b"Value1"
    );

    assert_eos!(
        p,
        h,
        b" Value2\r",
        HeaderLf1
    );

    assert_eq!(
        h.header_value,
        b"Value1 Value2"
    );
}

#[test]
fn multiple() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Value",
        HeaderValue
    );

    assert_eq!(
        h.header_value,
        b"Value"
    );

    assert_eos!(
        p,
        h,
        b"Time\r",
        HeaderLf1
    );

    assert_eq!(
        h.header_value,
        b"ValueTime"
    );
}

#[test]
fn single() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"ValueTime\r",
        HeaderLf1
    );

    assert_eq!(
        h.header_value,
        b"ValueTime"
    );
}

#[test]
fn space() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Value Time\r",
        HeaderLf1
    );

    assert_eq!(
        h.header_value,
        b"Value Time"
    );
}
