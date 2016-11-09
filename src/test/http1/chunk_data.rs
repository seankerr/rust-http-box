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
use test::http1::*;

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        $parser.init_chunked();

        assert_eos!($parser, $handler,
                    b"F;extension1=value1\r\n",
                    ParserState::ChunkData);
    });
}

#[test]
fn byte_check() {
    for byte in 0..255 {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        assert_eos!(p, h,
                    &[byte],
                    ParserState::ChunkData);
    }
}

#[test]
fn multiple() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"abcdefg",
                ParserState::ChunkData);
    assert_eq!(h.chunk_data, b"abcdefg");
    assert_eos!(p, h,
                b"hijklmno",
                ParserState::ChunkDataNewline1);
    assert_eq!(h.chunk_data, b"abcdefghijklmno");
}

#[test]
fn multiple_chunks() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"abcdefghijklmno\r\n",
                ParserState::ChunkLength1);
    assert_eq!(h.chunk_data, b"abcdefghijklmno");
    assert_eos!(p, h,
                b"5\r\n",
                ParserState::ChunkData);
    assert_eos!(p, h,
                b"pqrst",
                ParserState::ChunkDataNewline1);
    assert_eq!(h.chunk_data, b"abcdefghijklmnopqrst");
}

#[test]
fn single() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"abcdefghijklmno",
                ParserState::ChunkDataNewline1);
    assert_eq!(h.chunk_data, b"abcdefghijklmno");
}
