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
use test::http1::*;

macro_rules! setup {
    () => ({
        let mut handler = DebugHandler::new();
        let mut parser  = Parser::new();

        assert_eos!(
            parser,
            handler,
            b"GET / HTTP/1.1\r\nFieldName: Value",
            HeaderValue
        );

        (parser, handler)
    });
}

#[test]
fn callback_exit() {
    struct CallbackHandler;

    impl HttpHandler for CallbackHandler {
        fn on_headers_finished(&mut self) -> bool {
            false
        }
    }

    let mut h = CallbackHandler;
    let mut p = Parser::new();

    assert_eos!(
        p,
        h,
        b"GET / HTTP/1.1\r\nFieldName: Value",
        HeaderValue
    );

    assert_callback!(
        p,
        h,
        b"\r\n\r\n",
        Finished
    );
}

#[test]
fn finished() {
    let (mut p, mut h) = setup!();

    assert_finished!(
        p,
        h,
        b"\r\n\r\n"
    );

    assert!(h.headers_finished);
}
