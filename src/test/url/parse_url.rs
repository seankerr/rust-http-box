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

use test::*;
use url::*;

macro_rules! url {
    ($stream:expr,
     $scheme:expr, $userinfo:expr, $hostname:expr, $ipv4:expr, $ipv6:expr, $port:expr, $path:expr,
     $query_string:expr, $fragment:expr,
     $has_scheme:expr, $has_userinfo:expr, $has_hostname:expr, $has_ipv4:expr, $has_ipv6:expr,
     $has_port:expr, $has_path:expr, $has_query_string:expr, $has_fragment:expr, $length:expr) => ({
        let mut fragment         = vec![];
        let mut has_fragment     = false;
        let mut has_ipv4         = false;
        let mut has_ipv6         = false;
        let mut has_hostname     = false;
        let mut has_path         = false;
        let mut has_port         = false;
        let mut has_query_string = false;
        let mut has_scheme       = false;
        let mut has_userinfo     = false;
        let mut hostname         = vec![];
        let mut ipv4             = vec![];
        let mut ipv6             = vec![];
        let mut path             = vec![];
        let mut port             = 0;
        let mut query_string     = vec![];
        let mut scheme           = vec![];
        let mut userinfo         = vec![];

        assert!(match parse_url($stream,
                                |segment| {
                                    match segment {
                                        UrlSegment::Fragment(x) => {
                                            has_fragment = true;
                                            fragment.extend_from_slice(x);
                                        },
                                        UrlSegment::Host(host) => {
                                            match host {
                                                Host::Hostname(x) => {
                                                    has_hostname = true;
                                                    hostname.extend_from_slice(x);
                                                },
                                                Host::IPv4(x) => {
                                                    has_ipv4 = true;
                                                    ipv4.extend_from_slice(x);
                                                },
                                                Host::IPv6(x) => {
                                                    has_ipv6 = true;
                                                    ipv6.extend_from_slice(x);
                                                }
                                            }
                                        },
                                        UrlSegment::Path(x) => {
                                            has_path = true;
                                            path.extend_from_slice(x);
                                        },
                                        UrlSegment::Port(x) => {
                                            has_port = true;
                                            port = x;
                                        },
                                        UrlSegment::QueryString(x) => {
                                            has_query_string = true;
                                            query_string.extend_from_slice(x);
                                        },
                                        UrlSegment::Scheme(x) => {
                                            has_scheme = true;
                                            scheme.extend_from_slice(x);
                                        },
                                        UrlSegment::UserInfo(x) => {
                                            has_userinfo = true;
                                            userinfo.extend_from_slice(x);
                                        }
                                    }
                                }) {
            Ok($length) => {
                assert_eq!(fragment, $fragment);
                assert_eq!(hostname, $hostname);
                assert_eq!(ipv4, $ipv4);
                assert_eq!(ipv6, $ipv6);
                assert_eq!(path, $path);
                assert_eq!(port, $port);
                assert_eq!(query_string, $query_string);
                assert_eq!(scheme, $scheme);
                assert_eq!(userinfo, $userinfo);
                assert_eq!(has_fragment, $has_fragment);
                assert_eq!(has_hostname, $has_hostname);
                assert_eq!(has_ipv4, $has_ipv4);
                assert_eq!(has_ipv6, $has_ipv6);
                assert_eq!(has_path, $has_path);
                assert_eq!(has_port, $has_port);
                assert_eq!(has_query_string, $has_query_string);
                assert_eq!(has_scheme, $has_scheme);
                assert_eq!(has_userinfo, $has_userinfo);
                true
            },
            _ => false
        });
    });
}

macro_rules! url_error {
    ($stream:expr, $error:path, $byte:expr) => ({
        assert!(match parse_url($stream, |_| {}) {
            Err($error(x)) => {
                assert_eq!(x, $byte);
                true
            },
            _ => false
        });
    });
}

#[test]
fn fragment() {
    url!(b"#fragment",
         b"", b"", b"", b"", b"", 0, b"", b"", b"fragment",
         false, false, false, false, false, false, false, false, true, 9);
}

#[test]
fn fragment_byte_check() {
    // invalid bytes
    loop_non_visible(b"", |byte| {
        url_error!(&[b'/', b'#', byte], UrlError::Fragment, byte);
    });

    // valid bytes
    loop_visible(b"", |byte| {
        url!(&[b'/', b'#', byte],
             b"", b"", b"", b"", b"", 0, b"/", b"", &[byte],
             false, false, false, false, false, false, true, false, true, 3);
    });
}

#[test]
fn path() {
    url!(b"/path",
         b"", b"", b"", b"", b"", 0, b"/path", b"", b"",
         false, false, false, false, false, false, true, false, false, 5);
}

#[test]
fn path_byte_check() {
    // invalid bytes
    loop_non_visible(b"", |byte| {
        url_error!(&[b'/', byte], UrlError::Path, byte);
    });

    // valid bytes
    loop_visible(b"/?#", |byte| {
        url!(&[b'/', byte],
             b"", b"", b"", b"", b"", 0, &[b'/', byte], b"", b"",
             false, false, false, false, false, false, true, false, false, 2);
    });
}

#[test]
fn path_query_string() {
    url!(b"/path?query-string",
         b"", b"", b"", b"", b"", 0, b"/path", b"query-string", b"",
         false, false, false, false, false, false, true, true, false, 18);
}

#[test]
fn path_fragment() {
    url!(b"/path#fragment-data",
         b"", b"", b"", b"", b"", 0, b"/path", b"", b"fragment-data",
         false, false, false, false, false, false, true, false, true, 19);
}

#[test]
fn path_query_string_fragment() {
    url!(b"/path?query-string#fragment-data",
         b"", b"", b"", b"", b"", 0, b"/path", b"query-string", b"fragment-data",
         false, false, false, false, false, false, true, true, true, 32);
}

#[test]
fn query_string() {
    url!(b"?query-string",
         b"", b"", b"", b"", b"", 0, b"", b"query-string", b"",
         false, false, false, false, false, false, false, true, false, 13);
}

#[test]
fn query_string_byte_check() {
    // invalid bytes
    loop_non_visible(b"", |byte| {
        url_error!(&[b'/', b'?', byte], UrlError::QueryString, byte);
    });

    // valid bytes
    loop_visible(b"#", |byte| {
        url!(&[b'/', b'?', byte],
             b"", b"", b"", b"", b"", 0, b"/", &[byte], b"",
             false, false, false, false, false, false, true, true, false, 3);
    });
}

#[test]
fn scheme() {
    url!(b"http://",
         b"http", b"", b"", b"", b"", 0, b"", b"", b"",
         true, false, false, false, false, false, false, false, false, 7);
}

#[test]
fn scheme_error1() {
    url_error!(b"0http://", UrlError::Scheme, b'0');
}

#[test]
fn scheme_error2() {
    url_error!(b"http$://", UrlError::Scheme, b'$');
}
