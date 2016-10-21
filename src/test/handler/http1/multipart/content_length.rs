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
                                              Content-Length: 14F\r\n\
                                              \r\n\
                                              Multipart Data\r\n\
                                              --BOUNDARY--") {
        Ok(Success::Finished(63)) => {
            assert_eq!(None, h.content_length());
            true
        },
        _ => false
    });
}

#[test]
fn content_length_multiple_ok() {
    let mut h = MultipartHandler::new(b"BOUNDARY");
    let mut p = Parser::new();

    assert!(match p.parse_multipart(&mut h, b"--BOUNDARY\r\n\
                                              Content-Length: 14\r\n\
                                              \r\n\
                                              Multipart") {
        Ok(Success::Eos(43)) => {
            assert_eq!(Some(14), h.content_length());
            assert_eq!(p.state(), ParserState::MultipartDataByLength);
            true
        },
        _ => false
    });

    assert!(match p.parse_multipart(&mut h, b" Data") {
        Ok(Success::Eos(5)) => {
            assert_eq!(p.state(), ParserState::MultipartDataNewline1);
            true
        },
        _ => false
    });

    assert!(match p.parse_multipart(&mut h, b"\r\n\
                                              --BOUNDARY\r\n") {
        Ok(Success::Eos(14)) => true,
        _ => false
    });

    assert!(match p.parse_multipart(&mut h, b"Content-Length: 13\r\n\
                                              \r\n\
                                              Hello, world!\r\n\
                                              --BOUNDARY--") {
        Ok(Success::Finished(49)) => {
            assert_eq!(Some(13), h.content_length());
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
                                              Content-Length: 14\r\n\
                                              \r\n\
                                              Multipart Data") {
        Ok(Success::Eos(48)) => {
            assert_eq!(Some(14), h.content_length());
            assert_eq!(p.state(), ParserState::MultipartDataNewline1);
            true
        },
        _ => false
    });

    assert!(match p.parse_multipart(&mut h, b"\r\n\
                                              --BOUNDARY\r\n") {
        Ok(Success::Eos(14)) => true,
        _ => false
    });

    assert!(match p.parse_multipart(&mut h, b"Content-Length: 13\r\n\
                                              \r\n\
                                              Hello, world!\r\n\
                                              --BOUNDARY--") {
        Ok(Success::Finished(49)) => {
            assert_eq!(Some(13), h.content_length());
            true
        },
        _ => false
    });
}
