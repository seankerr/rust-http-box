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
    () => ({
        let mut parser = Parser::new_chunked(DebugHandler::new());

        assert_eos!(parser,
                    b"F;extension1=value1\r\n",
                    ChunkData);

        parser
    });
}

#[test]
fn byte_check() {
    for byte in 0..255 {
        let mut p = setup!();

        assert_eos!(p,
                    &[byte],
                    ChunkData);
    }
}

#[test]
fn multiple() {
    let mut p = setup!();

    assert_eos!(p,
                b"abcdefg",
                ChunkData);

    assert_eq!(p.handler().chunk_data,
               b"abcdefg");

    assert_eos!(p,
                b"hijklmno",
                ChunkDataNewline1);

    assert_eq!(p.handler().chunk_data,
               b"abcdefghijklmno");
}

#[test]
fn multiple_chunks() {
    let mut p = setup!();

    assert_eos!(p,
                b"abcdefghijklmno\r\n",
                ChunkLength1);

    assert_eq!(p.handler().chunk_data,
               b"abcdefghijklmno");

    assert_eos!(p,
                b"5\r\n",
                ChunkData);

    assert_eos!(p,
                b"pqrst",
                ChunkDataNewline1);

    assert_eq!(p.handler().chunk_data,
               b"abcdefghijklmnopqrst");
}

#[test]
fn single() {
    let mut p = setup!();

    assert_eos!(p,
                b"abcdefghijklmno",
                ChunkDataNewline1);

    assert_eq!(p.handler().chunk_data,
               b"abcdefghijklmno");
}
