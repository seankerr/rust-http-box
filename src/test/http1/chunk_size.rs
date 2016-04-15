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
    size: u64
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
}

#[test]
fn chunk_size_valid() {
    let mut h = H{size: 0};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nFF\r") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.size, 0xFF);
    assert_eq!(p.get_state(), State::ChunkNewline);
}

#[test]
fn chunk_size_maximum() {
    let mut h = H{size: 0};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nFFFFFFFFFFFFFFFF\r") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.size, 0xFFFFFFFFFFFFFFFF);
    assert_eq!(p.get_state(), State::ChunkNewline);
}

#[test]
fn chunk_size_too_long() {
    let mut h = H{size: 0};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nFFFFFFFFFFFFFFFF0\r") {
        Err(ParserError::MaxChunkSizeLength(_,_)) => true,
        _                                         => false
    });
}

#[test]
fn chunk_size_invalid() {
    let mut h = H{size: 0};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nG") {
        Err(ParserError::ChunkSize(_,_)) => true,
        _                                => false
    });
}

#[test]
fn chunk_size_invalid_crlf() {
    let mut h = H{size: 0};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nF\rinvalid") {
        Err(ParserError::CrlfSequence(_)) => true,
        _                                 => false
    });
}
