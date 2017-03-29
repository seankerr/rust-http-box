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

        assert_eos(
            &mut p,
            &mut h,
            b"GET / HTTP/1.1\r\n\
              Header:",
            ParserState::StripHeaderValue,
            b"GET / HTTP/1.1\r\n\
              Header:".len()
        );

        (p, h)
    });
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
            ParserState::HeaderEscapedValue,
            b"escaped \"\\".len()
        );

        assert_eos(
            &mut p,
            &mut h,
            &[b],
            ParserState::HeaderQuotedValue,
            [b].len()
        );
    }
}

#[test]
fn allowed_header_fields() {
    // skip `\t` and `space`, otherwise state will change
    for b in header_field_vec().iter()
                               .filter(|&x| *x != b' ')
                               .filter(|&x| *x != b'\t') {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::HeaderValue,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            b"\r",
            ParserState::HeaderLf1,
            b"\r".len()
        );

        assert_eq!(
            h.header_value[0],
            *b
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
            ParserState::HeaderQuotedValue,
            b"\"".len()
        );

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::HeaderQuotedValue,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            b"\"",
            ParserState::HeaderValue,
            b"\"".len()
        );

        assert_eq!(
            h.header_value.len(),
            3
        );

        assert_eq!(
            (&h.header_value)[0],
            b'\"'
        );

        assert_eq!(
            (&h.header_value)[1],
            *b
        );

        assert_eq!(
            (&h.header_value)[2],
            b'\"'
        );
    }
}

#[test]
fn entire_iter() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'"', ParserState::HeaderQuotedValue),
          (b'v', ParserState::HeaderQuotedValue),
          (b'a', ParserState::HeaderQuotedValue),
          (b'\\', ParserState::HeaderEscapedValue),
          (b'l', ParserState::HeaderQuotedValue),
          (b'u', ParserState::HeaderQuotedValue),
          (b'e', ParserState::HeaderQuotedValue),
          (b'1', ParserState::HeaderQuotedValue),
          (b'"', ParserState::HeaderValue),
          (b'v', ParserState::HeaderValue),
          (b'a', ParserState::HeaderValue),
          (b'l', ParserState::HeaderValue),
          (b'u', ParserState::HeaderValue),
          (b'e', ParserState::HeaderValue),
          (b'2', ParserState::HeaderValue),
          (b'\r', ParserState::HeaderLf1)]
    );

    assert_eq!(
        &h.header_value,
        b"\"va\\lue1\"value2"
    );
}

#[test]
fn multiline() {
    let (mut p, mut h) = setup!();

    assert_eos(
        &mut p,
        &mut h,
        b"Part1\r\n",
        ParserState::HeaderCr2,
        b"Part1\r\n".len()
    );

    assert_eos(
        &mut p,
        &mut h,
        b" Part2\r\n",
        ParserState::HeaderCr2,
        b" Part2\r\n".len()
    );

    assert_eos(
        &mut p,
        &mut h,
        b" Part3\r\n",
        ParserState::HeaderCr2,
        b" Part3\r\n".len()
    );

    assert_eq!(
        &h.header_value,
        b"Part1Part2Part3"
    )
}

#[test]
fn not_allowed_escaped_error() {
    // escaped data can only be 7bit non-control
    for b in (0..255).filter(|&x| x < 0x20 || x > 0x7A) {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            b"escaped \"\\",
            ParserState::HeaderEscapedValue,
            b"escaped \"\\".len()
        );

        assert_error(
            &mut p,
            &mut h,
            &[b],
            ParserError::HeaderValue(b)
        );
    }
}

#[test]
fn not_allowed_header_fields_error() {
    // skip `\r` and `"`, otherwise state will change
    for b in (0..255).filter(|&x| !is_header_field(x))
                     .filter(|&x| x != b'\r')
                     .filter(|&x| x != b'"') {
        let (mut p, mut h) = setup!();

        assert_error(
            &mut p,
            &mut h,
            &[b],
            ParserError::HeaderValue(b)
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
            ParserState::HeaderQuotedValue,
            b"\"".len()
        );

        assert_error(
            &mut p,
            &mut h,
            &[b],
            ParserError::HeaderValue(b)
        );
    }
}

#[test]
fn state_finished() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'V', ParserState::LowerHeaderName),
          (b'\r', ParserState::HeaderLf1),
          (b'\n', ParserState::HeaderCr2),
          (b'\r', ParserState::HeaderLf2)]
    );

    assert_finished(
        &mut p,
        &mut h,
        b"\n",
        b"\n".len()
    );
}
