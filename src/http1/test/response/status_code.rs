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
use test::digit_vec;

macro_rules! setup {
    () => ({
        let (mut p, mut h) = http1_setup!();

        assert_eos(
            &mut p,
            &mut h,
            b"HTTP/1.1 ",
            ParserState::ResponseStatusCode1,
            b"HTTP/1.1 ".len()
        );

        (p, h)
    });
}

#[test]
fn allowed() {
    for b in digit_vec().iter() {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::ResponseStatusCode2,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::ResponseStatusCode3,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::ResponseStatusCodeSpace,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            b" ",
            ParserState::ResponseStatus1,
            b" ".len()
        );

        assert_eq!(
            h.status_code,
            (((*b - b'0') as u16 * 100) + ((*b - b'0') as u16 * 10) + (*b - b'0') as u16)
        )
    }
}

#[test]
fn callback_exit() {
    struct H;
    impl HttpHandler for H {
        fn on_status_code(&mut self, _: u16) -> bool {
            false
        }
    }

    let mut h = H;
    let mut p = Parser::new();

    assert_callback(
        &mut p,
        &mut h,
        b"HTTP/1.1 200 ",
        ParserState::ResponseStatus1,
        b"HTTP/1.1 200 ".len()
    );
}

#[test]
fn entire_iter() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'2', ParserState::ResponseStatusCode2),
          (b'0', ParserState::ResponseStatusCode3),
          (b'0', ParserState::ResponseStatusCodeSpace),
          (b' ', ParserState::ResponseStatus1)]
    );

    assert_eq!(
        h.status_code,
        200
    );
}

#[test]
fn not_allowed_error1() {
    // skip `\t` and `space`, otherwise state will change
    for b in (0..255).filter(|&x| !is_digit!(x))
                     .filter(|&x| x != b'\t')
                     .filter(|&x| x != b' ') {
        let (mut p, mut h) = setup!();

        assert_error(
            &mut p,
            &mut h,
            &[b],
            ParserError::StatusCode(b)
        );
    }
}

#[test]
fn not_allowed_error2() {
    // skip `\t` and `space`, otherwise state will change
    for b in (0..255).filter(|&x| !is_digit!(x))
                     .filter(|&x| x != b'\t')
                     .filter(|&x| x != b' ') {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            b"1",
            ParserState::ResponseStatusCode2,
            b"1".len()
        );

        assert_error(
            &mut p,
            &mut h,
            &[b],
            ParserError::StatusCode(b)
        );
    }
}

#[test]
fn not_allowed_error3() {
    // skip `\t` and `space`, otherwise state will change
    for b in (0..255).filter(|&x| !is_digit!(x))
                     .filter(|&x| x != b'\t')
                     .filter(|&x| x != b' ') {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            b"12",
            ParserState::ResponseStatusCode3,
            b"12".len()
        );

        assert_error(
            &mut p,
            &mut h,
            &[b],
            ParserError::StatusCode(b)
        );
    }
}

#[test]
fn state_response_status1() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'2', ParserState::ResponseStatusCode2),
          (b'0', ParserState::ResponseStatusCode3),
          (b'0', ParserState::ResponseStatusCodeSpace),
          (b' ', ParserState::ResponseStatus1)]
    );
}
