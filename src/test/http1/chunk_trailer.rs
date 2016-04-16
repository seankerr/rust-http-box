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
use std::str;

struct H {
    field: Vec<u8>,
    size:  u64,
    value: Vec<u8>
}

impl HttpHandler for H {
    fn get_transfer_encoding(&mut self) -> TransferEncoding {
        println!("get_transfer_encoding: chunked");
        TransferEncoding::Chunked
    }

    fn on_chunk_size(&mut self, size: u64) -> bool {
        println!("on_chunk_size: {}", size);
        self.size = size;
        true
    }

    fn on_header_field(&mut self, data: &[u8]) -> bool {
        println!("on_header_field: {:?}", str::from_utf8(data).unwrap());
        self.field.extend_from_slice(data);
        true
    }

    fn on_header_value(&mut self, data: &[u8]) -> bool {
        println!("on_header_value: {:?}", str::from_utf8(data).unwrap());
        self.value.extend_from_slice(data);
        true
    }
}

#[test]
fn trailer_single() {
    let mut h = H{field: Vec::new(), size: 0, value: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nD\r\nHello, world!\r\n0\r\nTrailer: Value\r\n\r\n") {
        Ok(_) => true,
        _     => false
    });

    assert_eq!(h.field, b"Trailer");
    assert_eq!(h.value, b"Value");
    assert_eq!(p.get_state(), State::Finished);
}

#[test]
fn trailer_multiple() {
    let mut h = H{field: Vec::new(), size: 0, value: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nD\r\nHello, world!\r\n0\r\nTrailer1: Value1\r\n") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.field, b"Trailer1");
    assert_eq!(h.value, b"Value1");
    assert_eq!(p.get_state(), State::Newline3);

    assert!(match p.parse(&mut h, b"Trailer2: Value2\r\n\r\n") {
        Ok(_) => true,
        _     => false
    });

    assert_eq!(h.field, b"Trailer1Trailer2");
    assert_eq!(h.value, b"Value1Value2");
    assert_eq!(p.get_state(), State::Finished);
}

#[test]
fn trailer_no_trailers() {
    let mut h = H{field: Vec::new(), size: 0, value: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nD\r\nHello, world!\r\n0\r\n\r\n") {
        Ok(_) => true,
        _     => false
    });

    assert_eq!(p.get_state(), State::Finished);
}
