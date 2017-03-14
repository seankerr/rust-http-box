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
        let mut parser  = Parser::new_head();

        assert_eos!(
            parser,
            handler,
            b"HTTP/1.0 ",
            StripResponseStatusCode
        );

        (parser, handler)
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_digits(b" \t", |byte| {
        let (mut p, mut h) = setup!();

        assert_error_byte!(
            p,
            h,
            &[byte],
            StatusCode,
            byte
        );
    });

    // valid bytes
    loop_digits(b"", |byte| {
        let (mut p, mut h) = setup!();

        assert_eos!(
            p,
            h,
            &[byte],
            ResponseStatusCode
        );
    });
}

#[test]
fn callback_exit() {
    struct CallbackHandler;

    impl HttpHandler for CallbackHandler {
        fn on_status_code(&mut self, _code: u16) -> bool {
            false
        }
    }

    let mut h = CallbackHandler;
    let mut p = Parser::new_head();

    assert_eos!(
        p,
        h,
        b"HTTP/1.0 ",
        StripResponseStatusCode
    );

    assert_callback!(
        p,
        h,
        b"100 ",
        StripResponseStatus,
        3
    );
}

#[test]
fn v0 () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"0 ",
        StripResponseStatus
    );

    assert_eq!(
        h.status_code,
        0
    );
}

#[test]
fn v999 () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"999 ",
        StripResponseStatus
    );

    assert_eq!(
        h.status_code,
        999
    );
}

#[test]
fn v1000 () {
    let (mut p, mut h) = setup!();

    assert_error_byte!(
        p,
        h,
        b"1000",
        StatusCode,
        b'0'
    );
}
