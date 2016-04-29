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

use Success;
use http1::*;
use url::*;

struct H {
    data: bool
}

impl HttpHandler for H {
    fn on_headers_finished(&mut self) -> bool {
        println!("on_headers_finished");
        self.data = true;
        true
    }
}

impl ParamHandler for H {}

#[test]
fn headers_finished_success() {
    let mut h = H{data: false};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nField: Value\r\n\r\n") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert!(h.data);
    assert_eq!(p.get_state(), State::Body);
}

#[test]
fn headers_finished_fail() {
    let mut h = H{data: false};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nField: Value\r") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert!(!h.data);
    assert_eq!(p.get_state(), State::Newline2);

    p.reset();

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nField: Value\r\n") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert!(!h.data);
    assert_eq!(p.get_state(), State::Newline3);

    p.reset();

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nField: Value\n") {
        Err(ParserError::CrlfSequence(_,_)) => true,
        _ => false
    });

    assert!(!h.data);
    assert_eq!(p.get_state(), State::Dead);
}
