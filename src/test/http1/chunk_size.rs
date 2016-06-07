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
use test::http1::*;

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        $handler.set_transfer_encoding(TransferEncoding::Chunked);

        setup(&mut $parser, &mut $handler, b"GET / HTTP/1.1\r\n\r\n", State::ChunkSize);
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_hex(b"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        if let ParserError::ChunkSize(x) = assert_error(&mut p, &mut h, &[byte]).unwrap() {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid bytes
    loop_hex(b"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        assert_eos(&mut p, &mut h, &[byte], State::ChunkSize, 1);
    });
}

#[test]
fn callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn get_transfer_encoding(&mut self) -> TransferEncoding {
            TransferEncoding::Chunked
        }

        fn on_chunk_size(&mut self, _size: u64) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new();

    setup(&mut p, &mut h, b"GET / HTTP/1.1\r\n\r\n", State::ChunkSize);

    assert_callback(&mut p, &mut h, b"F\r", State::ChunkSizeNewline, 2);
}

#[test]
fn missing_size() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    if let ParserError::ChunkSize(x) = assert_error(&mut p, &mut h,
                                                    b"\r").unwrap() {
        assert_eq!(x, b'\r');
    } else {
        panic!();
    }
}

#[test]
fn size1() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"F\r", State::ChunkSizeNewline, 2);
    assert_eq!(h.chunk_size, 15);
}

#[test]
fn size2() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"FF\r", State::ChunkSizeNewline, 3);
    assert_eq!(h.chunk_size, 255);
}

#[test]
fn size3() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"FFF\r", State::ChunkSizeNewline, 4);
    assert_eq!(h.chunk_size, 4095);
}

#[test]
fn size4() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"FFFF\r", State::ChunkSizeNewline, 5);
    assert_eq!(h.chunk_size, 65535);
}

#[test]
fn size5() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"FFFFF\r", State::ChunkSizeNewline, 6);
    assert_eq!(h.chunk_size, 1048575);
}

#[test]
fn size6() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"FFFFFF\r", State::ChunkSizeNewline, 7);
    assert_eq!(h.chunk_size, 16777215);
}

#[test]
fn size7() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"FFFFFFF\r", State::ChunkSizeNewline, 8);
    assert_eq!(h.chunk_size, 268435455);
}

#[test]
fn size8() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"FFFFFFFF\r", State::ChunkSizeNewline, 9);
    assert_eq!(h.chunk_size, 4294967295);
}

#[test]
fn size9() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"FFFFFFFFF\r", State::ChunkSizeNewline, 10);
    assert_eq!(h.chunk_size, 68719476735);
}

#[test]
fn size10() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"FFFFFFFFFF\r", State::ChunkSizeNewline, 11);
    assert_eq!(h.chunk_size, 1099511627775);
}

#[test]
fn too_long() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    if let ParserError::ChunkSize(x) = assert_error(&mut p, &mut h,
                                                    b"FFFFFFFFFF0").unwrap() {
        assert_eq!(x, b'0');
    } else {
        panic!();
    }
}
