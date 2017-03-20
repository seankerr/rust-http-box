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
    () => ({
        let mut handler = DebugHandler::new();
        let mut parser  = Parser::new();

        assert_eos!(
            parser,
            handler,
            b"HTTP/1.1 200 ",
            StripResponseStatus
        );

        (parser, handler)
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_tokens(
        b"\r\t ",
        |byte| {
            let (mut p, mut h) = setup!();

            assert_error_byte!(
                p,
                h,
                &[byte],
                Status,
                byte
            );
        }
    );

    // valid bytes
    loop_tokens(
        b"",
        |byte| {
            let (mut p, mut h) = setup!();

            assert_eos!(
                p,
                h,
                &[byte],
                ResponseStatus
            );
        }
    );
}

#[test]
fn callback_exit() {
    struct CallbackHandler;

    impl HttpHandler for CallbackHandler {
        fn on_status(&mut self, _status: &[u8]) -> bool {
            false
        }
    }

    let mut h = CallbackHandler;
    let mut p = Parser::new();

    assert_eos!(
        p,
        h,
        b"HTTP/1.1 200 ",
        StripResponseStatus
    );

    assert_callback!(
        p,
        h,
        b"A\tCOOL STATUS\r",
        InitialEnd
    );
}

#[test]
fn multiple() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"NOT ",
        ResponseStatus
    );

    assert_eq!(
        h.status,
        b"NOT "
    );

    assert_eos!(
        p,
        h,
        b"FOUND\r",
        PreHeadersLf1
    );

    assert_eq!(
        h.status,
        b"NOT FOUND"
    );
}

#[test]
fn single() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"NOT FOUND\r",
        PreHeadersLf1
    );

    assert_eq!(
        h.status,
        b"NOT FOUND"
    );
}
