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

        assert_eos(
            &mut p,
            &mut h,
            b"GET / HTTP/1.1\r\n",
            ParserState::HeaderCr2,
            b"GET / HTTP/1.1\r\n".len()
        );

        (p, h)
    });
}

#[test]
fn allowed_alpha_lowercase() {
    for b in alpha_lower_vec().iter() {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::LowerHeaderName,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            b":",
            ParserState::StripHeaderValue,
            b":".len()
        );

        assert_eq!(
            h.header_name,
            &[*b]
        );
    }
}

#[test]
fn allowed_alpha_uppercase() {
    for b in alpha_upper_vec().iter() {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::LowerHeaderName,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            b":",
            ParserState::StripHeaderValue,
            b":".len()
        );

        // header name is normalized to lower-case, so we need to lower-case
        assert_eq!(
            h.header_name,
            &[*b + 0x20]
        );
    }
}

#[test]
fn allowed_tokens_without_alpha() {
    for b in token_vec().iter().filter(|&x| !is_alpha!(*x)) {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::LowerHeaderName,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            b":",
            ParserState::StripHeaderValue,
            b":".len()
        );

        assert_eq!(
            h.header_name,
            &[*b]
        );
    }
}

#[test]
fn callback_exit() {
    struct H;
    impl HttpHandler for H {
        fn on_header_name(&mut self, _: &[u8]) -> bool {
            false
        }
    }

    let mut h = H;
    let mut p = Parser::new();

    assert_callback(
        &mut p,
        &mut h,
        b"X / HTTP/1.1\r\n\
          H",
        ParserState::LowerHeaderName,
        b"X / HTTP/1.1\r\n\
          H".len()
    );
}

#[test]
fn entire_iter() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'H', ParserState::LowerHeaderName),
          (b'e', ParserState::LowerHeaderName),
          (b'a', ParserState::LowerHeaderName),
          (b'd', ParserState::LowerHeaderName),
          (b'e', ParserState::LowerHeaderName),
          (b'r', ParserState::LowerHeaderName),
          (b':', ParserState::StripHeaderValue)]
    );

    assert_eq!(
        h.header_name,
        b"header"
    );
}

#[test]
fn not_allowed_error() {
    // skip `:` otherwise it will change state
    for b in non_token_vec().iter().filter(|&x| *x != b':') {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            b"X",
            ParserState::LowerHeaderName,
            b"X".len()
        );

        assert_error(
            &mut p,
            &mut h,
            &[*b],
            ParserError::HeaderName(*b)
        );
    }
}

#[test]
fn state_strip_header_value() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'H', ParserState::LowerHeaderName),
          (b':', ParserState::StripHeaderValue)]
    );
}
