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
use test::*;
use test::http1::*;

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_hex(b"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        if let ParserError::ChunkLength(x) = chunked_assert_error(&mut p, &mut h,
                                                                  &[byte]).unwrap() {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid bytes
    loop_hex(b"0", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        chunked_assert_eos(&mut p, &mut h, &[byte], ParserState::ChunkLength2, 1);
    });

    // starting 0 (end chunk)
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    chunked_assert_eos(&mut p, &mut h, b"0", ParserState::ChunkLengthCr, 1);
}

#[test]
fn callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn on_chunk_length(&mut self, _length: usize) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new();

    chunked_assert_callback(&mut p, &mut h, b"F\r", ParserState::ChunkLengthLf, 2);
}

#[test]
fn missing_length() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    if let ParserError::ChunkLength(x) = chunked_assert_error(&mut p, &mut h,
                                                              b"\r").unwrap() {
        assert_eq!(x, b'\r');
    } else {
        panic!();
    }
}

#[test]
fn length1() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    chunked_assert_eos(&mut p, &mut h, b"F\r", ParserState::ChunkLengthLf, 2);
    assert_eq!(h.chunk_length, 15);
}

#[test]
fn length2() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    chunked_assert_eos(&mut p, &mut h, b"FF\r", ParserState::ChunkLengthLf, 3);
    assert_eq!(h.chunk_length, 255);
}

#[test]
fn length3() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    chunked_assert_eos(&mut p, &mut h, b"FFF\r", ParserState::ChunkLengthLf, 4);
    assert_eq!(h.chunk_length, 4095);
}

#[test]
fn too_long() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    if let ParserError::MaxChunkLength = chunked_assert_error(&mut p, &mut h,
                                                              b"FFFFFFFFFFFFFFFF0").unwrap() {
    } else {
        panic!();
    }
}
