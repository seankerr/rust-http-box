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

#[test]
fn on_chunk_begin() {
    struct H;
    impl HttpHandler for H {
        fn on_chunk_begin(&mut self) -> bool {
            false
        }
    }

    let mut h = H;
    let mut p = Parser::new();

    p.init_chunked();

    assert_callback(
        &mut p,
        &mut h,
        b"F",
        ParserState::ChunkLength2,
        b"F".len()
    );
}

#[test]
fn on_chunk_data() {
    struct H;
    impl HttpHandler for H {
        fn on_chunk_data(&mut self, _: &[u8]) -> bool {
            false
        }
    }

    let mut h = H;
    let mut p = Parser::new();

    p.init_chunked();

    assert_eos(
        &mut p,
        &mut h,
        b"F\r\n",
        ParserState::ChunkData,
        b"F\r\n".len()
    );

    assert_callback(
        &mut p,
        &mut h,
        b"X",
        ParserState::ChunkData,
        b"X".len()
    );
}

#[test]
fn on_chunk_extension_finished() {
    struct H;
    impl HttpHandler for H {
        fn on_chunk_extension_finished(&mut self) -> bool {
            false
        }
    }

    let mut h = H;
    let mut p = Parser::new();

    p.init_chunked();

    assert_callback(
        &mut p,
        &mut h,
        b"F;extension;",
        ParserState::StripChunkExtensionName,
        b"F;extension;".len()
    );
}

#[test]
fn on_chunk_extension_name() {
    struct H;
    impl HttpHandler for H {
        fn on_chunk_extension_name(&mut self, _: &[u8]) -> bool {
            false
        }
    }

    let mut h = H;
    let mut p = Parser::new();

    p.init_chunked();

    assert_callback(
        &mut p,
        &mut h,
        b"F;extension",
        ParserState::LowerChunkExtensionName,
        b"F;extension".len()
    );
}

#[test]
fn on_chunk_extension_value() {
    struct H;
    impl HttpHandler for H {
        fn on_chunk_extension_value(&mut self, _: &[u8]) -> bool {
            false
        }
    }

    let mut h = H;
    let mut p = Parser::new();

    p.init_chunked();

    assert_callback(
        &mut p,
        &mut h,
        b"F;extension=v",
        ParserState::ChunkExtensionValue,
        b"F;extension=v".len()
    );
}

#[test]
fn on_chunk_extensions_finished() {
    struct H;
    impl HttpHandler for H {
        fn on_chunk_extensions_finished(&mut self) -> bool {
            false
        }
    }

    let mut h = H;
    let mut p = Parser::new();

    p.init_chunked();

    assert_callback(
        &mut p,
        &mut h,
        b"F\r",
        ParserState::ChunkLengthLf,
        b"F\r".len()
    );
}

#[test]
fn on_chunk_length() {
    struct H;
    impl HttpHandler for H {
        fn on_chunk_length(&mut self, _: usize) -> bool {
            false
        }
    }

    let mut h = H;
    let mut p = Parser::new();

    p.init_chunked();

    assert_callback(
        &mut p,
        &mut h,
        b"F\r",
        ParserState::ChunkExtensionsFinished,
        b"F\r".len()
    );
}
