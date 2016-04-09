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

use http1::parser::*;

struct H {
    data: Vec<u8>
}

impl HttpHandler for H {
    fn on_url(&mut self, data: &[u8]) -> bool {
        self.data.extend_from_slice(data);
        true
    }
}

#[test]
fn request_uri_eof() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"GET /path") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"/path");
    assert_eq!(p.get_state(), State::RequestUrl);

    assert!(match p.parse(&mut h, b" ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(p.get_state(), State::RequestHttp1);
}
