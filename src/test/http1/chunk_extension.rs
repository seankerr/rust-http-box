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
    data: Vec<u8>,
    size: u64
}

impl HttpHandler for H {
    fn get_transfer_encoding(&mut self) -> TransferEncoding {
        println!("get_transfer_encoding: chunked");
        TransferEncoding::Chunked
    }

    fn on_chunk_extension(&mut self, extension: &[u8]) -> bool {
        println!("on_chunk_extension: {:?}", str::from_utf8(extension).unwrap());
        self.data.extend_from_slice(extension);
        true
    }

    fn on_chunk_size(&mut self, size: u64) -> bool {
        println!("on_chunk_size: {}", size);
        self.size = size;
        true
    }
}

impl ParamHandler for H {}

#[test]
fn chunk_extension_valid() {
    let mut h = H{data: Vec::new(), size: 0};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nFF;neat-extension\r") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"neat-extension");
    assert_eq!(h.size, 0xFF);
    assert_eq!(p.get_state(), State::ChunkSizeNewline2);
}

#[test]
fn chunk_extension_maximum() {
    let mut h = H{data: Vec::new(), size: 0};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nFF;neat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionnea\r") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data.len(), 255);
    assert_eq!(h.size, 0xFF);
    assert_eq!(p.get_state(), State::ChunkSizeNewline2);
}

#[test]
fn chunk_extension_too_long() {
    let mut h = H{data: Vec::new(), size: 0};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nFF;neat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat-extensionneat\r") {
        Err(ParserError::MaxChunkExtensionLength(_,_)) => true,
        _                                              => false
    });
}

#[test]
fn chunk_extension_illegal() {
    let mut h = H{data: Vec::new(), size: 0};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nD\r\nHello, world!\r\n0;neat-extension") {
        Err(ParserError::CrlfSequence(_,_)) => true,
        _                                   => false
    });
}
