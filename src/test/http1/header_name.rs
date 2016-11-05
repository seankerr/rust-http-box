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
    ($parser:expr, $handler:expr) => ({
        setup(&mut $parser, &mut $handler, b"GET / HTTP/1.1\r\n", ParserState::PreHeadersCr2);
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_tokens(b"\r\n \t:", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        if let ParserError::HeaderName(x) = assert_error(&mut p, &mut h, &[byte]).unwrap() {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid non-alphabetical bytes
    loop_tokens(b"\r\n \t:", |byte| {
        if !is_alpha!(byte) {
            let mut h = DebugHandler::new();
            let mut p = Parser::new();

            setup!(p, h);

            assert_eos(&mut p, &mut h, &[byte], ParserState::LowerHeaderName, 1);
        }
    });

    // valid lower-cased alphabetical bytes
    loop_tokens(b"\r\n \t:", |byte| {
        if byte > 0x60 && byte < 0x7B {
            let mut h = DebugHandler::new();
            let mut p = Parser::new();

            setup!(p, h);

            assert_eos(&mut p, &mut h, &[byte], ParserState::LowerHeaderName, 1);
        }
    });

    // valid upper-cased alphabetical bytes
    loop_tokens(b"\r\n \t:", |byte| {
        if byte > 0x40 && byte < 0x5B {
            let mut h = DebugHandler::new();
            let mut p = Parser::new();

            setup!(p, h);

            assert_eos(&mut p, &mut h, &[byte], ParserState::LowerHeaderName, 1);
        }
    });
}

#[test]
fn by_name_accept() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Accept:                   ", ParserState::StripHeaderValue, 26);
}

#[test]
fn by_name_accept_charset() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Accept-Charset:           ", ParserState::StripHeaderValue, 26);
}

#[test]
fn by_name_accept_encoding() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Accept-Encoding:          ", ParserState::StripHeaderValue, 26);
}

#[test]
fn by_name_accept_language() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Accept-Language:          ", ParserState::StripHeaderValue, 26);
}

#[test]
fn by_name_authorization() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Authorization:            ", ParserState::StripHeaderValue, 26);
}

#[test]
fn by_name_connection() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Connection:               ", ParserState::StripHeaderValue, 26);
}

#[test]
fn by_name_content_type() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Content-Type:             ", ParserState::StripHeaderValue, 26);
}

#[test]
fn by_name_content_length() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Content-Length:           ", ParserState::StripHeaderValue, 26);
}
#[test]
fn by_name_cookie() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Cookie:                   ", ParserState::StripHeaderValue, 26);
}
#[test]
fn by_name_cache_control() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Cache-Control:            ", ParserState::StripHeaderValue, 26);
}
#[test]
fn by_name_content_security_policy() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Content-Security-Policy:  ", ParserState::StripHeaderValue, 26);
}
#[test]
fn by_name_location() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Location:                 ", ParserState::StripHeaderValue, 26);
}
#[test]
fn by_name_last_modified() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Last-Modified:            ", ParserState::StripHeaderValue, 26);
}
#[test]
fn by_name_pragma() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Pragma:                   ", ParserState::StripHeaderValue, 26);
}
#[test]
fn by_name_set_cookie() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Set-Cookie:               ", ParserState::StripHeaderValue, 26);
}

#[test]
fn by_name_transfer_encoding() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Transfer-Encoding:        ", ParserState::StripHeaderValue, 26);
}

#[test]
fn by_name_user_agent() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"User-Agent:               ", ParserState::StripHeaderValue, 26);
}
#[test]
fn by_name_upgrade() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"Upgrade:                  ", ParserState::StripHeaderValue, 26);
}

#[test]
fn by_name_x_powered_by() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"X-Powered-By:             ", ParserState::StripHeaderValue, 26);
}

#[test]
fn by_name_x_forwarded_for() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"X-Forwarded-For:          ", ParserState::StripHeaderValue, 26);
}

#[test]
fn by_name_x_forwarded_host() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"X-Forwarded-Host:         ", ParserState::StripHeaderValue, 26);
}

#[test]
fn by_name_x_xss_protection() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"X-XSS-Protection:         ", ParserState::StripHeaderValue, 26);
}

#[test]
fn by_name_x_webkit_csp() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"X-WebKit-CSP:             ", ParserState::StripHeaderValue, 26);
}

#[test]
fn by_name_www_authenticate() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"WWW-Authenticate:         ", ParserState::StripHeaderValue, 26);
}

#[test]
fn callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn on_header_name(&mut self, _field: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new();

    setup!(p, h);

    assert_callback(&mut p, &mut h, b"F", ParserState::LowerHeaderName, 1);
}

#[test]
fn multiple() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"F", ParserState::LowerHeaderName, 1);
    assert_eq!(h.header_name, b"f");
    assert_eos(&mut p, &mut h, b"i", ParserState::LowerHeaderName, 1);
    assert_eq!(h.header_name, b"fi");
    assert_eos(&mut p, &mut h, b"e", ParserState::LowerHeaderName, 1);
    assert_eq!(h.header_name, b"fie");
    assert_eos(&mut p, &mut h, b"l", ParserState::LowerHeaderName, 1);
    assert_eq!(h.header_name, b"fiel");
    assert_eos(&mut p, &mut h, b"d", ParserState::LowerHeaderName, 1);
    assert_eq!(h.header_name, b"field");
    assert_eos(&mut p, &mut h, b"N", ParserState::LowerHeaderName, 1);
    assert_eq!(h.header_name, b"fieldn");
    assert_eos(&mut p, &mut h, b"a", ParserState::LowerHeaderName, 1);
    assert_eq!(h.header_name, b"fieldna");
    assert_eos(&mut p, &mut h, b"m", ParserState::LowerHeaderName, 1);
    assert_eq!(h.header_name, b"fieldnam");
    assert_eos(&mut p, &mut h, b"e", ParserState::LowerHeaderName, 1);
    assert_eq!(h.header_name, b"fieldname");
    assert_eos(&mut p, &mut h, b":", ParserState::StripHeaderValue, 1);
}

#[test]
fn normalize() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"HEADER-FIELD", ParserState::LowerHeaderName, 12);
    assert_eq!(h.header_name, b"header-field");
}

#[test]
fn single() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"FieldName:", ParserState::StripHeaderValue, 10);
    assert_eq!(h.header_name, b"fieldname");
}
