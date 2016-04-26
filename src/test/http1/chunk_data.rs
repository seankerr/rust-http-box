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
use std::str;

struct H {
    data: Vec<u8>
}

impl HttpHandler for H {
    fn get_transfer_encoding(&mut self) -> TransferEncoding {
        println!("get_transfer_encoding: chunked");
        TransferEncoding::Chunked
    }

    fn on_chunk_data(&mut self, data: &[u8]) -> bool {
        println!("on_chunk_data: {:?}", str::from_utf8(data).unwrap());
        self.data.extend_from_slice(data);
        true
    }
}

impl ParamHandler for H {}

#[test]
fn chunk_data_valid() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nD\r\nHello, world!") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"Hello, world!");
    assert_eq!(p.get_state(), State::ChunkDataNewline1);
}

#[test]
fn chunk_data_multiple_pieces() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nD\r\nHello,") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"Hello,");
    assert_eq!(p.get_state(), State::ChunkData);

    assert!(match p.parse(&mut h, b" world!") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"Hello, world!");
    assert_eq!(p.get_state(), State::ChunkDataNewline1);
}

#[test]
fn chunk_data_crlf() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nD\r\nHello, world!x") {
        Err(ParserError::CrlfSequence(_)) => true,
        _                                 => false
    });

    p.reset();

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nD\r\nHello, world!\rx") {
        Err(ParserError::CrlfSequence(_)) => true,
        _                                 => false
    });
}
