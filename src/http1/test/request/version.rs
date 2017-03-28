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

        // bypass method
        assert_eos(
            &mut p,
            &mut h,
            b"GET /path HTTP/",
            ParserState::RequestVersionMajor1,
            b"GET /path HTTP/".len()
        );

        (p, h)
    });
}

#[test]
fn allowed1() {
    for b in digit_vec().iter() {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::RequestVersionMajor2,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            b".",
            ParserState::RequestVersionMinor1,
            b".".len()
        );

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::RequestVersionMinor2,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            b"\r",
            ParserState::InitialLf,
            b"\r".len()
        );

        assert_eq!(
            h.version_major,
            (*b - b'0') as u16
        );

        assert_eq!(
            h.version_minor,
            (*b - b'0') as u16
        );
    }
}

#[test]
fn allowed2() {
    for b in digit_vec().iter() {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::RequestVersionMajor2,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::RequestVersionMajor3,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            b".",
            ParserState::RequestVersionMinor1,
            b".".len()
        );

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::RequestVersionMinor2,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::RequestVersionMinor3,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            b"\r",
            ParserState::InitialLf,
            b"\r".len()
        );

        assert_eq!(
            h.version_major,
            (((*b - b'0') * 10) + (*b - b'0')) as u16
        );

        assert_eq!(
            h.version_minor,
            (((*b - b'0') * 10) + (*b - b'0')) as u16
        );
    }
}

#[test]
fn allowed3() {
    for b in digit_vec().iter() {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::RequestVersionMajor2,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::RequestVersionMajor3,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::RequestVersionPeriod,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            b".",
            ParserState::RequestVersionMinor1,
            b".".len()
        );

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::RequestVersionMinor2,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::RequestVersionMinor3,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::RequestVersionCr,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            b"\r",
            ParserState::InitialLf,
            b"\r".len()
        );

        assert_eq!(
            h.version_major,
            (((*b - b'0') as u16 * 100) + ((*b - b'0') as u16 * 10) + (*b - b'0') as u16)
        );

        assert_eq!(
            h.version_minor,
            (((*b - b'0') as u16 * 100) + ((*b - b'0') as u16 * 10) + (*b - b'0') as u16)
        );
    }
}

#[test]
fn callback_exit() {
    struct H;
    impl HttpHandler for H {
        fn on_version(&mut self, _: u16, _: u16) -> bool {
            false
        }
    }

    let mut h = H;
    let mut p = Parser::new();

    assert_callback(
        &mut p,
        &mut h,
        b"X / HTTP/1.1\r",
        ParserState::InitialEnd,
        b"X / HTTP/1.1\r".len()
    );
}

#[test]
fn entire_iter() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'1', ParserState::RequestVersionMajor2),
          (b'.', ParserState::RequestVersionMinor1),
          (b'1', ParserState::RequestVersionMinor2),
          (b'\r', ParserState::InitialLf)]
    );

    assert_eq!(
        h.version_major,
        1
    );

    assert_eq!(
        h.version_minor,
        1
    );
}

#[test]
fn major_not_allowed_error() {
    for b in non_digit_vec().iter() {
        let (mut p, mut h) = setup!();

        assert_error(
            &mut p,
            &mut h,
            &[*b],
            ParserError::Version(*b)
        );
    }
}

#[test]
fn major_overflow_error() {
    let (mut p, mut h) = setup!();

    assert_error(
        &mut p,
        &mut h,
        b"9990",
        ParserError::Version(b'0')
    );
}

#[test]
fn minor_not_allowed_error() {
    for b in non_digit_vec().iter() {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            b"1.",
            ParserState::RequestVersionMinor1,
            b"1.".len()
        );

        assert_error(
            &mut p,
            &mut h,
            &[*b],
            ParserError::Version(*b)
        );
    }
}

#[test]
fn minor_overflow_error() {
    let (mut p, mut h) = setup!();

    assert_error(
        &mut p,
        &mut h,
        b"1.9990",
        ParserError::Version(b'0')
    );
}

#[test]
fn state_lower_header_name() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'1', ParserState::RequestVersionMajor2),
          (b'.', ParserState::RequestVersionMinor1),
          (b'1', ParserState::RequestVersionMinor2),
          (b'\r', ParserState::InitialLf),
          (b'\n', ParserState::HeaderCr2),
          (b'H', ParserState::LowerHeaderName)]
    );
}
