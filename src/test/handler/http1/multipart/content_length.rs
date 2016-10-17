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

use fsm::*;
use handler::*;
use http1::*;
use test::http1::*;

#[test]
fn content_length_non_byte_error() {
    let mut h = MultipartHandler::new(b"BOUNDARY");
    let mut p = Parser::new();

    assert!(match p.parse_multipart(&mut h, b"--BOUNDARY\r\n\
                                              Content-Length: 123F\r\n\
                                              \r\n\
                                              \r\n\
                                              --BOUNDARY--\r\n") {
        Ok(Success::Finished(52)) => {
            assert_eq!(None, h.content_length());
            true
        },
        _ => false
    });
}

#[test]
fn content_length_ok() {
    let mut h = MultipartHandler::new(b"BOUNDARY");
    let mut p = Parser::new();

    assert!(match p.parse_multipart(&mut h, b"--BOUNDARY\r\n\
                                              Content-Length: 1234\r\n\
                                              \r\n\
                                              \r\n\
                                              --BOUNDARY--\r\n") {
        Ok(Success::Finished(52)) => {
            assert_eq!(Some(1234), h.content_length());
            true
        },
        _ => false
    });
}
