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
use test::*;

macro_rules! setup {
    () => ({
        let (mut p, mut h) = http1_setup!();

        p.init_chunked();

        assert_eos(
            &mut p,
            &mut h,
            b"F;",
            ParserState::StripChunkExtensionName,
            b"F;".len()
        );

        (p, h)
    });
}

#[test]
fn allowed() {
    for b in token_vec().iter() {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::LowerChunkExtensionName,
            [b].len()
        );
    }
}

#[test]
fn callback() {
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
fn entire_iter() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'E', ParserState::LowerChunkExtensionName),
          (b'x', ParserState::LowerChunkExtensionName),
          (b't', ParserState::LowerChunkExtensionName),
          (b'e', ParserState::LowerChunkExtensionName),
          (b'N', ParserState::LowerChunkExtensionName),
          (b's', ParserState::LowerChunkExtensionName),
          (b'I', ParserState::LowerChunkExtensionName),
          (b'o', ParserState::LowerChunkExtensionName),
          (b'N', ParserState::LowerChunkExtensionName),
          (b'\r', ParserState::ChunkLengthLf)]
    );

    assert_eq!(
        &h.chunk_extension_name,
        b"extension"
    );
}

#[test]
fn not_allowed() {
    // skip `\r`, `\t`, `space`, `=`, and `;`, otherwise state will change
    for b in non_token_vec().iter()
                            .filter(|&x| *x != b'\r')
                            .filter(|&x| *x != b'\t')
                            .filter(|&x| *x != b' ')
                            .filter(|&x| *x != b'=')
                            .filter(|&x| *x != b';') {
        let (mut p, mut h) = setup!();

        assert_error(
            &mut p,
            &mut h,
            &[*b],
            ParserError::ChunkExtensionName(*b)
        );
    }
}

#[test]
fn state_chunk_extensions_finished() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'E', ParserState::LowerChunkExtensionName),
          (b'\r', ParserState::ChunkExtensionsFinished)]
    );
}

#[test]
fn state_strip_chunk_extension_name1() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'E', ParserState::LowerChunkExtensionName),
          (b';', ParserState::StripChunkExtensionName)]
    );
}

#[test]
fn state_strip_chunk_extension_name2() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b' ', ParserState::StripChunkExtensionName),
          (b'\t', ParserState::StripChunkExtensionName)]
    );
}

#[test]
fn state_strip_chunk_extension_value() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'E', ParserState::LowerChunkExtensionName),
          (b'=', ParserState::StripChunkExtensionValue)]
    );
}
