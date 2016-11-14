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
use test::*;
use test::http1::*;

macro_rules! setup {
    () => ({
        let mut parser = Parser::new_head(DebugHandler::new());

        assert_eos!(parser,
                    b"GET / HTTP/1.1\r\n",
                    PreHeadersCr2);

        parser
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_tokens(b"\r\n \t:", |byte| {
        let mut p = setup!();

        assert_error_byte!(p,
                           &[byte],
                           HeaderName,
                           byte);
    });

    // valid non-alphabetical bytes
    loop_tokens(b"\r\n \t:", |byte| {
        if !is_alpha!(byte) {
            let mut p = setup!();

            assert_eos!(p,
                        &[byte],
                        LowerHeaderName);
        }
    });

    // valid lower-cased alphabetical bytes
    loop_tokens(b"\r\n \t:", |byte| {
        if byte > 0x60 && byte < 0x7B {
            let mut p = setup!();

            assert_eos!(p,
                        &[byte],
                        LowerHeaderName);
        }
    });

    // valid upper-cased alphabetical bytes
    loop_tokens(b"\r\n \t:", |byte| {
        if byte > 0x40 && byte < 0x5B {
            let mut p = setup!();

            assert_eos!(p,
                        &[byte],
                        LowerHeaderName);
        }
    });
}

#[test]
fn by_name_accept() {
    let mut p = setup!();

    assert_eos!(p,
                b"Accept:                   ",
                StripHeaderValue);
}

#[test]
fn by_name_accept_charset() {
    let mut p = setup!();

    assert_eos!(p,
                b"Accept-Charset:           ",
                StripHeaderValue);
}

#[test]
fn by_name_accept_encoding() {
    let mut p = setup!();

    assert_eos!(p,
                b"Accept-Encoding:          ",
                StripHeaderValue);
}

#[test]
fn by_name_accept_language() {
    let mut p = setup!();

    assert_eos!(p,
                b"Accept-Language:          ",
                StripHeaderValue);
}

#[test]
fn by_name_authorization() {
    let mut p = setup!();

    assert_eos!(p,
                b"Authorization:            ",
                StripHeaderValue);
}

#[test]
fn by_name_connection() {
    let mut p = setup!();

    assert_eos!(p,
                b"Connection:               ",
                StripHeaderValue);
}

#[test]
fn by_name_content_type() {
    let mut p = setup!();

    assert_eos!(p,
                b"Content-Type:             ",
                StripHeaderValue);
}

#[test]
fn by_name_content_length() {
    let mut p = setup!();

    assert_eos!(p,
                b"Content-Length:           ",
                StripHeaderValue);
}
#[test]
fn by_name_cookie() {
    let mut p = setup!();

    assert_eos!(p,
                b"Cookie:                   ",
                StripHeaderValue);
}
#[test]
fn by_name_cache_control() {
    let mut p = setup!();

    assert_eos!(p,
                b"Cache-Control:            ",
                StripHeaderValue);
}
#[test]
fn by_name_content_security_policy() {
    let mut p = setup!();

    assert_eos!(p,
                b"Content-Security-Policy:  ",
                StripHeaderValue);
}
#[test]
fn by_name_location() {
    let mut p = setup!();

    assert_eos!(p,
                b"Location:                 ",
                StripHeaderValue);
}
#[test]
fn by_name_last_modified() {
    let mut p = setup!();

    assert_eos!(p,
                b"Last-Modified:            ",
                StripHeaderValue);
}
#[test]
fn by_name_pragma() {
    let mut p = setup!();

    assert_eos!(p,
                b"Pragma:                   ",
                StripHeaderValue);
}
#[test]
fn by_name_set_cookie() {
    let mut p = setup!();

    assert_eos!(p,
                b"Set-Cookie:               ",
                StripHeaderValue);
}

#[test]
fn by_name_transfer_encoding() {
    let mut p = setup!();

    assert_eos!(p,
                b"Transfer-Encoding:        ",
                StripHeaderValue);
}

#[test]
fn by_name_user_agent() {
    let mut p = setup!();

    assert_eos!(p,
                b"User-Agent:               ",
                StripHeaderValue);
}
#[test]
fn by_name_upgrade() {
    let mut p = setup!();

    assert_eos!(p,
                b"Upgrade:                  ",
                StripHeaderValue);
}

#[test]
fn by_name_x_powered_by() {
    let mut p = setup!();

    assert_eos!(p,
                b"X-Powered-By:             ",
                StripHeaderValue);
}

#[test]
fn by_name_x_forwarded_for() {
    let mut p = setup!();

    assert_eos!(p,
                b"X-Forwarded-For:          ",
                StripHeaderValue);
}

#[test]
fn by_name_x_forwarded_host() {
    let mut p = setup!();

    assert_eos!(p,
                b"X-Forwarded-Host:         ",
                StripHeaderValue);
}

#[test]
fn by_name_x_xss_protection() {
    let mut p = setup!();

    assert_eos!(p,
                b"X-XSS-Protection:         ",
                StripHeaderValue);
}

#[test]
fn by_name_x_webkit_csp() {
    let mut p = setup!();

    assert_eos!(p,
                b"X-WebKit-CSP:             ",
                StripHeaderValue);
}

#[test]
fn by_name_www_authenticate() {
    let mut p = setup!();

    assert_eos!(p,
                b"WWW-Authenticate:         ",
                StripHeaderValue);
}

#[test]
fn callback_exit() {
    struct CallbackHandler;

    impl HttpHandler for CallbackHandler {
        fn on_header_name(&mut self, _field: &[u8]) -> bool {
            false
        }
    }

    let mut p = Parser::new_head(CallbackHandler);

    assert_eos!(p,
                b"GET / HTTP/1.1\r\n",
                PreHeadersCr2);

    assert_callback!(p,
                     b"F",
                     LowerHeaderName);
}

#[test]
fn multiple() {
    let mut p = setup!();

    assert_eos!(p,
                b"F",
                LowerHeaderName);

    assert_eq!(p.handler().header_name,
               b"f");

    assert_eos!(p,
                b"i",
                LowerHeaderName);

    assert_eq!(p.handler().header_name,
               b"fi");

    assert_eos!(p,
                b"e",
                LowerHeaderName);

    assert_eq!(p.handler().header_name,
               b"fie");

    assert_eos!(p,
                b"l",
                LowerHeaderName);

    assert_eq!(p.handler().header_name,
               b"fiel");

    assert_eos!(p,
                b"d",
                LowerHeaderName);

    assert_eq!(p.handler().header_name,
               b"field");

    assert_eos!(p,
                b"N",
                LowerHeaderName);

    assert_eq!(p.handler().header_name,
               b"fieldn");

    assert_eos!(p,
                b"a",
                LowerHeaderName);

    assert_eq!(p.handler().header_name,
               b"fieldna");

    assert_eos!(p,
                b"m",
                LowerHeaderName);

    assert_eq!(p.handler().header_name,
               b"fieldnam");

    assert_eos!(p,
                b"e",
                LowerHeaderName);

    assert_eq!(p.handler().header_name,
               b"fieldname");

    assert_eos!(p,
                b":",
                StripHeaderValue);
}

#[test]
fn normalize() {
    let mut p = setup!();

    assert_eos!(p,
                b"HEADER-FIELD",
                LowerHeaderName);

    assert_eq!(p.handler().header_name,
               b"header-field");
}

#[test]
fn single() {
    let mut p = setup!();

    assert_eos!(p,
                b"FieldName:",
                StripHeaderValue);

    assert_eq!(p.handler().header_name,
               b"fieldname");
}
