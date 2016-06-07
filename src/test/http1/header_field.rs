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

use handler::*;
use http1::*;
use test::*;
use test::http1::*;

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        setup(&mut $parser, &mut $handler, b"GET / HTTP/1.1\r\n", State::PreHeaders2);
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_tokens(b"\r\n \t:", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        if let ParserError::HeaderField(x) = assert_error(&mut p, &mut h, &[byte]).unwrap() {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid bytes
    loop_tokens(b"\r\n \t:", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        assert_eos(&mut p, &mut h, &[byte], State::HeaderField, 1);
    });
}

#[test]
fn by_name_accept() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Accept:                   ", State::StripHeaderValue, 26);
}

#[test]
fn by_name_accept_charset() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Accept-Charset:           ", State::StripHeaderValue, 26);
}

#[test]
fn by_name_accept_encoding() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Accept-Encoding:          ", State::StripHeaderValue, 26);
}

#[test]
fn by_name_accept_language() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Accept-Language:          ", State::StripHeaderValue, 26);
}

#[test]
fn by_name_authorization() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Authorization:            ", State::StripHeaderValue, 26);
}

#[test]
fn by_name_connection() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Connection:               ", State::StripHeaderValue, 26);
}

#[test]
fn by_name_content_type() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Content-Type:             ", State::StripHeaderValue, 26);
}

#[test]
fn by_name_content_length() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Content-Length:           ", State::StripHeaderValue, 26);
}
#[test]
fn by_name_cookie() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Cookie:                   ", State::StripHeaderValue, 26);
}
#[test]
fn by_name_cache_control() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Cache-Control:            ", State::StripHeaderValue, 26);
}
#[test]
fn by_name_content_security_policy() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Content-Security-Policy:  ", State::StripHeaderValue, 26);
}
#[test]
fn by_name_location() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Location:                 ", State::StripHeaderValue, 26);
}
#[test]
fn by_name_last_modified() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Last-Modified:            ", State::StripHeaderValue, 26);
}
#[test]
fn by_name_pragma() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Pragma:                   ", State::StripHeaderValue, 26);
}
#[test]
fn by_name_set_cookie() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Set-Cookie:               ", State::StripHeaderValue, 26);
}

#[test]
fn by_name_transfer_encoding() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Transfer-Encoding:        ", State::StripHeaderValue, 26);
}

#[test]
fn by_name_user_agent() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"User-Agent:               ", State::StripHeaderValue, 26);
}
#[test]
fn by_name_upgrade() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Upgrade:                  ", State::StripHeaderValue, 26);
}

#[test]
fn by_name_x_powered_by() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"X-Powered-By:             ", State::StripHeaderValue, 26);
}

#[test]
fn by_name_x_forwarded_for() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"X-Forwarded-For:          ", State::StripHeaderValue, 26);
}

#[test]
fn by_name_x_forwarded_host() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"X-Forwarded-Host:         ", State::StripHeaderValue, 26);
}

#[test]
fn by_name_x_xss_protection() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"X-XSS-Protection:         ", State::StripHeaderValue, 26);
}

#[test]
fn by_name_x_webkit_csp() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"X-WebKit-CSP:             ", State::StripHeaderValue, 26);
}

#[test]
fn by_name_x_content_security_policy() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"X-Content-Security-Policy:", State::StripHeaderValue, 26);
}

#[test]
fn by_name_www_authenticate() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"WWW-Authenticate:         ", State::StripHeaderValue, 26);
}

#[test]
fn callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn on_header_field(&mut self, _field: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new();

    setup!(p, h);

    assert_callback(&mut p, &mut h, b"F", State::HeaderField, 1);
}

#[test]
fn multiple() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"F", State::HeaderField, 1);
    assert_eq!(h.header_field, b"F");
    assert_eos(&mut p, &mut h, b"i", State::HeaderField, 1);
    assert_eq!(h.header_field, b"Fi");
    assert_eos(&mut p, &mut h, b"e", State::HeaderField, 1);
    assert_eq!(h.header_field, b"Fie");
    assert_eos(&mut p, &mut h, b"l", State::HeaderField, 1);
    assert_eq!(h.header_field, b"Fiel");
    assert_eos(&mut p, &mut h, b"d", State::HeaderField, 1);
    assert_eq!(h.header_field, b"Field");
    assert_eos(&mut p, &mut h, b"N", State::HeaderField, 1);
    assert_eq!(h.header_field, b"FieldN");
    assert_eos(&mut p, &mut h, b"a", State::HeaderField, 1);
    assert_eq!(h.header_field, b"FieldNa");
    assert_eos(&mut p, &mut h, b"m", State::HeaderField, 1);
    assert_eq!(h.header_field, b"FieldNam");
    assert_eos(&mut p, &mut h, b"e", State::HeaderField, 1);
    assert_eq!(h.header_field, b"FieldName");
    assert_eos(&mut p, &mut h, b":", State::StripHeaderValue, 1);
}

#[test]
fn single() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"FieldName:", State::StripHeaderValue, 10);
    assert_eq!(h.header_field, b"FieldName");
}
