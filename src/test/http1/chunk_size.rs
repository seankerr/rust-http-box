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

use handler::*;
use http1::*;
use test::*;

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        $handler.set_transfer_encoding(TransferEncoding::Chunked);

        setup(&mut $parser, &mut $handler, b"GET / HTTP/1.1\r\n\r\n", State::ChunkSize1);
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_hex(b"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        setup!(p, h);

        if let ParserError::ChunkSize(_,x) = assert_error(&mut p, &mut h, &[byte]).unwrap() {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid bytes
    loop_hex(b"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        setup!(p, h);

        assert_eof(&mut p, &mut h, &[byte], State::ChunkSize2, 1);
    });
}

#[test]
fn chunk_size1() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"F\r", State::ChunkSizeNewline, 2);
    assert_eq!(h.chunk_size, 15);
}

#[test]
fn chunk_size2() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"FF\r", State::ChunkSizeNewline, 3);
    assert_eq!(h.chunk_size, 255);
}

#[test]
fn chunk_size3() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"FFF\r", State::ChunkSizeNewline, 4);
    assert_eq!(h.chunk_size, 4095);
}

#[test]
fn chunk_size4() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"FFFF\r", State::ChunkSizeNewline, 5);
    assert_eq!(h.chunk_size, 65535);
}

#[test]
fn chunk_size5() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"FFFFF\r", State::ChunkSizeNewline, 6);
    assert_eq!(h.chunk_size, 1048575);
}

#[test]
fn chunk_size6() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"FFFFFF\r", State::ChunkSizeNewline, 7);
    assert_eq!(h.chunk_size, 16777215);
}

#[test]
fn chunk_size7() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"FFFFFFF\r", State::ChunkSizeNewline, 8);
    assert_eq!(h.chunk_size, 268435455);
}

#[test]
fn chunk_size8() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"FFFFFFFF\r", State::ChunkSizeNewline, 9);
    assert_eq!(h.chunk_size, 4294967295);
}

#[test]
fn chunk_size9() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"FFFFFFFFF\r", State::ChunkSizeNewline, 10);
    assert_eq!(h.chunk_size, 68719476735);
}

#[test]
fn chunk_size10() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"FFFFFFFFFF\r", State::ChunkSizeNewline, 11);
    assert_eq!(h.chunk_size, 1099511627775);
}
