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
            b"GET ",
            ParserState::RequestUrl1,
            b"GET ".len()
        );

        (p, h)
    });
}

#[test]
fn allowed() {
    for b in visible_7bit_vec().iter() {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::RequestUrl2,
            [*b].len()
        );
    }
}

#[test]
fn callback_exit() {
    struct H;
    impl HttpHandler for H {
        fn on_url(&mut self, _: &[u8]) -> bool {
            false
        }
    }

    let mut h = H;
    let mut p = Parser::new();

    assert_callback(
        &mut p,
        &mut h,
        b"X /",
        ParserState::RequestUrl2,
        b"X /".len()
    );
}

#[test]
fn entire_iter() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'/', ParserState::RequestUrl2),
          (b'p', ParserState::RequestUrl2),
          (b'a', ParserState::RequestUrl2),
          (b't', ParserState::RequestUrl2),
          (b'h', ParserState::RequestUrl2),
          (b' ', ParserState::RequestHttp1)]
    );

    assert_eq!(
        h.url,
        b"/path"
    );
}

#[test]
fn not_allowed_error() {
    // skip `\t` and `space`, otherwise state will change
    for b in non_visible_7bit_vec().iter()
                                   .filter(|&x| *x != b'\t')
                                   .filter(|&x| *x != b' ') {
        let (mut p, mut h) = setup!();

        assert_error(
            &mut p,
            &mut h,
            &[*b],
            ParserError::Url(*b)
        );
    }
}

#[test]
fn state_request_http1() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'/', ParserState::RequestUrl2),
          (b' ', ParserState::RequestHttp1)]
    );
}
