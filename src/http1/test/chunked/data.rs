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

use http1::*;
use http1::test::*;

macro_rules! setup {
    () => ({
        let (mut p, mut h) = http1_setup!();

        p.init_chunked();

        assert_eos(
            &mut p,
            &mut h,
            b"C\r\n",
            ParserState::ChunkData,
            b"C\r\n".len()
        );

        (p, h)
    });
}

#[test]
fn entire_chunk() {
    let (mut p, mut h) = setup!();

    assert_eos(
        &mut p,
        &mut h,
        b"Hello, Rust!\r\n",
        ParserState::ChunkLength1,
        b"Hello, Rust!\r\n".len()
    );

    assert_eq!(
        &h.chunk_data,
        b"Hello, Rust!"
    );
}

#[test]
fn entire_iter() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'H', ParserState::ChunkData),
          (b'e', ParserState::ChunkData),
          (b'l', ParserState::ChunkData),
          (b'l', ParserState::ChunkData),
          (b'o', ParserState::ChunkData),
          (b',', ParserState::ChunkData),
          (b' ', ParserState::ChunkData),
          (b'R', ParserState::ChunkData),
          (b'u', ParserState::ChunkData),
          (b's', ParserState::ChunkData),
          (b't', ParserState::ChunkData),
          (b'!', ParserState::ChunkDataCr),
          (b'\r', ParserState::ChunkDataLf),
          (b'\n', ParserState::ChunkLength1)]
    );

    assert_eq!(
        &h.chunk_data,
        b"Hello, Rust!"
    );
}

#[test]
fn state_chunk_length1() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'H', ParserState::ChunkData),
          (b'e', ParserState::ChunkData),
          (b'l', ParserState::ChunkData),
          (b'l', ParserState::ChunkData),
          (b'o', ParserState::ChunkData),
          (b',', ParserState::ChunkData),
          (b' ', ParserState::ChunkData),
          (b'R', ParserState::ChunkData),
          (b'u', ParserState::ChunkData),
          (b's', ParserState::ChunkData),
          (b't', ParserState::ChunkData),
          (b'!', ParserState::ChunkData),
          (b'\r', ParserState::ChunkLengthLf),
          (b'\n', ParserState::ChunkLength1)]
    );
}
