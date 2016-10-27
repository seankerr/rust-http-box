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

#[test]
fn multiple() {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    h.headers_finished = false;

    assert!(match p.parse_chunked(&mut h, b"0\r\nField1: Value1\r\nField2: Value2\r\n\r\n") {
        Ok(Success::Finished(37)) => {
            assert!(h.headers_finished);
            assert_eq!(h.header_field, b"field1field2");
            assert_eq!(h.header_value, b"Value1Value2");
            true
        },
        _ => false
    });
}

#[test]
fn single() {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    h.headers_finished = false;

    assert!(match p.parse_chunked(&mut h, b"0\r\nField: Value\r\n\r\n") {
        Ok(Success::Finished(19)) => {
            assert!(h.headers_finished);
            assert_eq!(h.header_field, b"field");
            assert_eq!(h.header_value, b"Value");
            true
        },
        _ => false
    });
}
