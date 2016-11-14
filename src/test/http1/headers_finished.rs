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
        let mut parser = Parser::new_head(DebugHandler::new());

        assert_eos!(parser,
                    b"GET / HTTP/1.1\r\nFieldName: Value",
                    HeaderValue);

        parser
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

    let mut p = Parser::new_head(CallbackHandler);

    assert_eos!(p,
                b"GET / HTTP/1.1\r\nFieldName: Value",
                HeaderValue);

    assert_callback!(p,
                     b"\r\n\r\n",
                     Finished);
}

#[test]
fn finished() {
    let mut p = setup!();

    assert_finished!(p,
                     b"\r\n\r\n");

    assert!(p.handler().headers_finished);
}
