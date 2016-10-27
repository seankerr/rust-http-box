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

use extra::handler::*;
use fsm::*;
use http1::*;
use test::http1::*;

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        setup(&mut $parser, &mut $handler, b"GET / HTTP/1.1\r\nFieldName: Value\r\n",
              ParserState::Newline3);
    });
}

#[test]
fn content_length_non_byte_error() {
    let mut h = HeadersHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert!(match p.parse_head(&mut h, b"Content-Length: 938Q\r\n\
                                            \r\n", 0) {
        Ok(Success::Finished(24)) => {
            assert_eq!(None, h.content_length());
            true
        },
        _ => false
    });
}

#[test]
fn content_length_ok() {
    let mut h = HeadersHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert!(match p.parse_head(&mut h, b"Content-Length: 9382\r\n\
                                            \r\n", 0) {
        Ok(Success::Finished(24)) => {
            assert_eq!(Some(9382), h.content_length());
            true
        },
        _ => false
    });
}
