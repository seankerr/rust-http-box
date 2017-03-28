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

use byte::*;
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
            b"F;Extension=",
            ParserState::StripChunkExtensionValue,
            b"F;Extension=".len()
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
            ParserState::ChunkExtensionValue,
            [b].len()
        );
    }
}

#[test]
fn allowed_escaped() {
    // escaped data can only be 7bit non-control
    for b in (0..255).filter(|&x| x > 0x1F && x < 0x7B) {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            b"escaped \"\\",
            ParserState::ChunkExtensionEscapedValue,
            b"escaped \"\\".len()
        );

        assert_eos(
            &mut p,
            &mut h,
            &[b],
            ParserState::ChunkExtensionQuotedValue,
            [b].len()
        );
    }
}

#[test]
fn allowed_header_fields() {
    // skip `\t`, `;`, and `space`, otherwise state will change
    for b in header_field_vec().iter()
                               .filter(|&x| *x != b'\t')
                               .filter(|&x| *x != b';')
                               .filter(|&x| *x != b' ') {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::ChunkExtensionValue,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            b"\r",
            ParserState::ChunkLengthLf,
            b"\r".len()
        );

        assert_eq!(
            &h.chunk_extension_value,
            &[*b]
        );
    }
}

#[test]
fn allowed_quoted_header_fields() {
    // skip `"` and `\`, otherwise state will change
    for b in quoted_header_field_vec().iter()
                                      .filter(|&x| *x != b'"')
                                      .filter(|&x| *x != b'\\') {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            b"\"",
            ParserState::ChunkExtensionQuotedValue,
            b"\"".len()
        );

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::ChunkExtensionQuotedValue,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            b"\"",
            ParserState::ChunkExtensionValue,
            b"\"".len()
        );

        assert_eq!(
            &h.chunk_extension_value,
            &[*b]
        );
    }
}

#[test]
fn entire_iter() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'"', ParserState::ChunkExtensionQuotedValue),
          (b'v', ParserState::ChunkExtensionQuotedValue),
          (b'a', ParserState::ChunkExtensionQuotedValue),
          (b'\\', ParserState::ChunkExtensionEscapedValue),
          (b'l', ParserState::ChunkExtensionQuotedValue),
          (b'u', ParserState::ChunkExtensionQuotedValue),
          (b'e', ParserState::ChunkExtensionQuotedValue),
          (b'1', ParserState::ChunkExtensionQuotedValue),
          (b'"', ParserState::ChunkExtensionValue),
          (b'v', ParserState::ChunkExtensionValue),
          (b'a', ParserState::ChunkExtensionValue),
          (b'l', ParserState::ChunkExtensionValue),
          (b'u', ParserState::ChunkExtensionValue),
          (b'e', ParserState::ChunkExtensionValue),
          (b'2', ParserState::ChunkExtensionValue),
          (b'\r', ParserState::ChunkLengthLf)]
    );

    assert_eq!(
        &h.chunk_extension_value,
        b"value1value2"
    );
}

#[test]
fn not_allowed_header_fields_error() {
    // skip `\r`, `;` and `"`, otherwise state will change
    for b in (0..255).filter(|&x| !is_header_field(x))
                     .filter(|&x| x != b'\r')
                     .filter(|&x| x != b';')
                     .filter(|&x| x != b'"') {
        let (mut p, mut h) = setup!();

        assert_error(
            &mut p,
            &mut h,
            &[b],
            ParserError::ChunkExtensionValue(b)
        );
    }
}

#[test]
fn not_allowed_quoted_header_fields_error() {
    // skip `"` and `\`, otherwise state will change
    for b in (0..255).filter(|&x| !is_quoted_header_field(x))
                     .filter(|&x| x != b'"')
                     .filter(|&x| x != b'\\') {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            b"\"",
            ParserState::ChunkExtensionQuotedValue,
            b"\"".len()
        );

        assert_error(
            &mut p,
            &mut h,
            &[b],
            ParserError::ChunkExtensionValue(b)
        );
    }
}

#[test]
fn state_strip_chunk_extension_name() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'V', ParserState::ChunkExtensionValue),
          (b';', ParserState::StripChunkExtensionName)]
    );
}

#[test]
fn state_chunk_extension_escaped_value() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'V', ParserState::ChunkExtensionValue),
          (b'"', ParserState::ChunkExtensionQuotedValue),
          (b'\\', ParserState::ChunkExtensionEscapedValue)]
    );
}

#[test]
fn state_chunk_extension_quoted_value() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'V', ParserState::ChunkExtensionValue),
          (b'"', ParserState::ChunkExtensionQuotedValue)]
    );
}

#[test]
fn state_chunk_extension_value() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'V', ParserState::ChunkExtensionValue),
          (b'"', ParserState::ChunkExtensionQuotedValue),
          (b'V', ParserState::ChunkExtensionQuotedValue),
          (b'"', ParserState::ChunkExtensionValue)]
    );
}
