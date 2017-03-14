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
// | Author: Sean Kerr <sean@metatomic.io>                                                         |
// +-----------------------------------------------------------------------------------------------+

use http1::*;
use test::*;
use test::http1::*;

macro_rules! setup {
    () => ({
        let mut handler = DebugHandler::new();
        let mut parser  = Parser::new();

        assert_eos!(
            parser,
            handler,
            b"GET / HTTP/1.1\r\n",
            PreHeadersCr2
        );

        (parser, handler)
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_tokens(
        b"\r\n \t:",
        |byte| {
            let (mut p, mut h) = setup!();

            assert_error_byte!(
                p,
                h,
                &[byte],
                HeaderName,
                byte
            );
        }
    );

    // valid non-alphabetical bytes
    loop_tokens(
        b"\r\n \t:",
        |byte| {
            if !is_alpha!(byte) {
                let (mut p, mut h) = setup!();

                assert_eos!(
                    p,
                    h,
                    &[byte],
                    LowerHeaderName
                );
            }
        }
    );

    // valid lower-cased alphabetical bytes
    loop_tokens(
        b"",
        |byte| {
            if byte > 0x60 && byte < 0x7B {
                let (mut p, mut h) = setup!();

                assert_eos!(
                    p,
                    h,
                    &[byte],
                    LowerHeaderName
                );
            }
        }
    );

    // valid upper-cased alphabetical bytes
    loop_tokens(
        b"",
        |byte| {
            if byte > 0x40 && byte < 0x5B {
                let (mut p, mut h) = setup!();

                assert_eos!(
                    p,
                    h,
                    &[byte],
                    LowerHeaderName
                );
            }
        }
    );
}

#[test]
fn by_name_accept() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Accept:                   ",
        StripHeaderValue
    );
}

#[test]
fn by_name_accept_charset() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Accept-Charset:           ",
        StripHeaderValue
    );
}

#[test]
fn by_name_accept_encoding() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Accept-Encoding:          ",
        StripHeaderValue
    );
}

#[test]
fn by_name_accept_language() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Accept-Language:          ",
        StripHeaderValue
    );
}

#[test]
fn by_name_authorization() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Authorization:            ",
        StripHeaderValue
    );
}

#[test]
fn by_name_connection() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Connection:               ",
        StripHeaderValue
    );
}

#[test]
fn by_name_content_type() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Content-Type:             ",
        StripHeaderValue
    );
}

#[test]
fn by_name_content_length() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Content-Length:           ",
        StripHeaderValue
    );
}
#[test]
fn by_name_cookie() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Cookie:                   ",
        StripHeaderValue
    );
}
#[test]
fn by_name_cache_control() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Cache-Control:            ",
        StripHeaderValue
    );
}
#[test]
fn by_name_content_security_policy() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Content-Security-Policy:  ",
        StripHeaderValue
    );
}
#[test]
fn by_name_location() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Location:                 ",
        StripHeaderValue
    );
}
#[test]
fn by_name_last_modified() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Last-Modified:            ",
        StripHeaderValue
    );
}
#[test]
fn by_name_pragma() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Pragma:                   ",
        StripHeaderValue
    );
}
#[test]
fn by_name_set_cookie() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Set-Cookie:               ",
        StripHeaderValue
    );
}

#[test]
fn by_name_transfer_encoding() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Transfer-Encoding:        ",
        StripHeaderValue
    );
}

#[test]
fn by_name_user_agent() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"User-Agent:               ",
        StripHeaderValue
    );
}
#[test]
fn by_name_upgrade() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"Upgrade:                  ",
        StripHeaderValue
    );
}

#[test]
fn by_name_x_powered_by() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"X-Powered-By:             ",
        StripHeaderValue
    );
}

#[test]
fn by_name_x_forwarded_for() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"X-Forwarded-For:          ",
        StripHeaderValue
    );
}

#[test]
fn by_name_x_forwarded_host() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"X-Forwarded-Host:         ",
        StripHeaderValue
    );
}

#[test]
fn by_name_x_xss_protection() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"X-XSS-Protection:         ",
        StripHeaderValue
    );
}

#[test]
fn by_name_x_webkit_csp() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"X-WebKit-CSP:             ",
        StripHeaderValue
    );
}

#[test]
fn by_name_www_authenticate() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"WWW-Authenticate:         ",
        StripHeaderValue
    );
}

#[test]
fn callback_exit() {
    struct CallbackHandler;

    impl HttpHandler for CallbackHandler {
        fn on_header_name(&mut self, _field: &[u8]) -> bool {
            false
        }
    }

    let mut h = CallbackHandler;
    let mut p = Parser::new();

    assert_eos!(
        p,
        h,
        b"GET / HTTP/1.1\r\n",
        PreHeadersCr2
    );

    assert_callback!(
        p,
        h,
        b"F",
        LowerHeaderName
    );
}

#[test]
fn multiple() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"F",
        LowerHeaderName
    );

    assert_eq!(
        h.header_name,
        b"f"
    );

    assert_eos!(
        p,
        h,
        b"i",
        LowerHeaderName
    );

    assert_eq!(
        h.header_name,
        b"fi"
    );

    assert_eos!(
        p,
        h,
        b"e",
        LowerHeaderName
    );

    assert_eq!(
        h.header_name,
        b"fie"
    );

    assert_eos!(
        p,
        h,
        b"l",
        LowerHeaderName
    );

    assert_eq!(
        h.header_name,
        b"fiel"
    );

    assert_eos!(
        p,
        h,
        b"d",
        LowerHeaderName
    );

    assert_eq!(
        h.header_name,
        b"field"
    );

    assert_eos!(
        p,
        h,
        b"N",
        LowerHeaderName
    );

    assert_eq!(
        h.header_name,
        b"fieldn"
    );

    assert_eos!(
        p,
        h,
        b"a",
        LowerHeaderName
    );

    assert_eq!(
        h.header_name,
        b"fieldna"
    );

    assert_eos!(
        p,
        h,
        b"m",
        LowerHeaderName
    );

    assert_eq!(
        h.header_name,
        b"fieldnam"
    );

    assert_eos!(
        p,
        h,
        b"e",
        LowerHeaderName
    );

    assert_eq!(
        h.header_name,
        b"fieldname"
    );

    assert_eos!(
        p,
        h,
        b":",
        StripHeaderValue
    );
}

#[test]
fn normalize() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"HEADER-FIELD",
        LowerHeaderName
    );

    assert_eq!(
        h.header_name,
        b"header-field"
    );
}

#[test]
fn single() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"FieldName:",
        StripHeaderValue
    );

    assert_eq!(
        h.header_name,
        b"fieldname"
    );
}
