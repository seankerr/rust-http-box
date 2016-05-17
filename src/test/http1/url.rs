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

#[test]
fn empty() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert!(match p.parse_url(&mut h, b"") {
        Ok(Success::Finished(length)) => {
            assert_eq!(length, 0);
            assert_eq!(h.url_scheme, b"");
            assert_eq!(h.url_host, b"");
            assert_eq!(h.url_path, b"");
            assert_eq!(h.url_query_string, b"");
            assert_eq!(h.url_fragment, b"");
            true
        },
        _ => false
    });
}

#[test]
fn fragment_callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn on_url_fragment(&mut self, _fragment: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new_request();

    assert!(match p.parse_url(&mut h, b"http://www.rust-lang.org/path#fragment-data") {
        Ok(Success::Callback(length)) => {
            assert_eq!(length, 43);
            true
        },
        _ => false
    });
}

#[test]
fn fragment_error() {
    loop_non_visible(b"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        assert!(match p.parse_url(&mut h, &[b'h',b't',b't',b'p',b':',b'/',b'/',b'a',b'/',b'#',byte]) {
            Err(ParserError::UrlFragment(x)) => {
                assert_eq!(x, byte);
                true
            },
            _ => false
        });
    });
}

#[test]
fn host_callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn on_url_host(&mut self, _host: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new_request();

    assert!(match p.parse_url(&mut h, b"http://www.rust-lang.org") {
        Ok(Success::Callback(length)) => {
            assert_eq!(length, 24);
            true
        },
        _ => false
    });
}

#[test]
fn host_error() {
    loop_non_visible(b"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        assert!(match p.parse_url(&mut h, &[b'h',b't',b't',b'p',b':',b'/',b'/',byte]) {
            Err(ParserError::UrlHost(x)) => {
                assert_eq!(x, byte);
                true
            },
            _ => false
        });
    });
}

#[test]
fn path() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert!(match p.parse_url(&mut h, b"/path1/path2?param1=value1&param2=value2#fragment-data") {
        Ok(Success::Finished(length)) => {
            assert_eq!(length, 54);
            assert_eq!(h.url_scheme, b"");
            assert_eq!(h.url_host, b"");
            assert_eq!(h.url_path, b"/path1/path2");
            assert_eq!(h.url_query_string, b"param1=value1&param2=value2");
            assert_eq!(h.url_fragment, b"fragment-data");
            true
        },
        _ => false
    });
}

#[test]
fn path_callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn on_url_path(&mut self, _path: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new_request();

    assert!(match p.parse_url(&mut h, b"http://www.rust-lang.org/path") {
        Ok(Success::Callback(length)) => {
            assert_eq!(length, 29);
            true
        },
        _ => false
    });
}

#[test]
fn path_error() {
    loop_non_visible(b"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        assert!(match p.parse_url(&mut h, &[b'h',b't',b't',b'p',b':',b'/',b'/',b'a',b'/',byte]) {
            Err(ParserError::UrlPath(x)) => {
                assert_eq!(x, byte);
                true
            },
            _ => false
        });
    });
}

#[test]
fn path_no_fragment() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert!(match p.parse_url(&mut h, b"/path1/path2?param1=value1&param2=value2") {
        Ok(Success::Finished(length)) => {
            assert_eq!(length, 40);
            assert_eq!(h.url_scheme, b"");
            assert_eq!(h.url_host, b"");
            assert_eq!(h.url_path, b"/path1/path2");
            assert_eq!(h.url_query_string, b"param1=value1&param2=value2");
            assert_eq!(h.url_fragment, b"");
            true
        },
        _ => false
    });
}

#[test]
fn path_no_query_string() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert!(match p.parse_url(&mut h, b"/path1/path2#fragment-data") {
        Ok(Success::Finished(length)) => {
            assert_eq!(length, 26);
            assert_eq!(h.url_scheme, b"");
            assert_eq!(h.url_host, b"");
            assert_eq!(h.url_path, b"/path1/path2");
            assert_eq!(h.url_query_string, b"");
            assert_eq!(h.url_fragment, b"fragment-data");
            true
        },
        _ => false
    });
}

#[test]
fn path_no_query_string_no_fragment() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert!(match p.parse_url(&mut h, b"/path1/path2") {
        Ok(Success::Finished(length)) => {
            assert_eq!(length, 12);
            assert_eq!(h.url_scheme, b"");
            assert_eq!(h.url_host, b"");
            assert_eq!(h.url_path, b"/path1/path2");
            assert_eq!(h.url_query_string, b"");
            assert_eq!(h.url_fragment, b"");
            true
        },
        _ => false
    });
}

#[test]
fn query_string_callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn on_url_query_string(&mut self, _query_string: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new_request();

    assert!(match p.parse_url(&mut h, b"http://www.rust-lang.org/path?param1=value1") {
        Ok(Success::Callback(length)) => {
            assert_eq!(length, 43);
            true
        },
        _ => false
    });
}

#[test]
fn query_string_error() {
    loop_non_visible(b"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        assert!(match p.parse_url(&mut h, &[b'h',b't',b't',b'p',b':',b'/',b'/',b'a',b'/',b'?',byte]) {
            Err(ParserError::UrlQueryString(x)) => {
                assert_eq!(x, byte);
                true
            },
            _ => false
        });
    });
}

#[test]
fn scheme() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert!(match p.parse_url(&mut h, b"https://www.rust-lang.org/path1/path2?param1=value1&param2=value2#fragment-data") {
        Ok(Success::Finished(length)) => {
            assert_eq!(length, 79);
            assert_eq!(h.url_scheme, b"https");
            assert_eq!(h.url_host, b"www.rust-lang.org");
            assert_eq!(h.url_path, b"/path1/path2");
            assert_eq!(h.url_query_string, b"param1=value1&param2=value2");
            assert_eq!(h.url_fragment, b"fragment-data");
            true
        },
        _ => false
    });
}

#[test]
fn scheme_callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn on_url_scheme(&mut self, _scheme: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new_request();

    assert!(match p.parse_url(&mut h, b"http://") {
        Ok(Success::Callback(length)) => {
            assert_eq!(length, 5);
            true
        },
        _ => false
    });
}

#[test]
fn scheme_error() {
    loop_non_visible(b"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        assert!(match p.parse_url(&mut h, &[byte]) {
            Err(ParserError::UrlScheme(x)) => {
                assert_eq!(x, byte);
                true
            },
            _ => false
        });
    });
}

#[test]
fn scheme_no_fragment() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert!(match p.parse_url(&mut h, b"https://www.rust-lang.org/path1/path2?param1=value1&param2=value2") {
        Ok(Success::Finished(length)) => {
            assert_eq!(length, 65);
            assert_eq!(h.url_scheme, b"https");
            assert_eq!(h.url_host, b"www.rust-lang.org");
            assert_eq!(h.url_path, b"/path1/path2");
            assert_eq!(h.url_query_string, b"param1=value1&param2=value2");
            assert_eq!(h.url_fragment, b"");
            true
        },
        _ => false
    });
}

#[test]
fn scheme_no_path_no_query_string_no_fragment() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert!(match p.parse_url(&mut h, b"https://www.rust-lang.org") {
        Ok(Success::Finished(length)) => {
            assert_eq!(length, 25);
            assert_eq!(h.url_scheme, b"https");
            assert_eq!(h.url_host, b"www.rust-lang.org");
            assert_eq!(h.url_path, b"");
            assert_eq!(h.url_query_string, b"");
            assert_eq!(h.url_fragment, b"");
            true
        },
        _ => false
    });
}

#[test]
fn scheme_no_query_string() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert!(match p.parse_url(&mut h, b"https://www.rust-lang.org/path1/path2#fragment-data") {
        Ok(Success::Finished(length)) => {
            assert_eq!(length, 51);
            assert_eq!(h.url_scheme, b"https");
            assert_eq!(h.url_host, b"www.rust-lang.org");
            assert_eq!(h.url_path, b"/path1/path2");
            assert_eq!(h.url_query_string, b"");
            assert_eq!(h.url_fragment, b"fragment-data");
            true
        },
        _ => false
    });
}

#[test]
fn scheme_no_query_string_no_fragment() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert!(match p.parse_url(&mut h, b"https://www.rust-lang.org/path1/path2") {
        Ok(Success::Finished(length)) => {
            assert_eq!(length, 37);
            assert_eq!(h.url_scheme, b"https");
            assert_eq!(h.url_host, b"www.rust-lang.org");
            assert_eq!(h.url_path, b"/path1/path2");
            assert_eq!(h.url_query_string, b"");
            assert_eq!(h.url_fragment, b"");
            true
        },
        _ => false
    });
}
