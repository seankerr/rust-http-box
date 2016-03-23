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

use url::*;
use std::str;

struct H {
    fragment:     Vec<u8>,
    host:         Vec<u8>,
    path:         Vec<u8>,
    port:         u16,
    query_string: Vec<u8>,
    scheme:       Vec<u8>
}

impl UrlHandler for H {
    fn on_url_fragment(&mut self, data: &[u8]) -> bool {
        println!("on_url_fragment: {:?}", str::from_utf8(data).unwrap());
        self.fragment.extend_from_slice(data);
        true
    }

    fn on_url_host(&mut self, data: &[u8]) -> bool {
        println!("on_url_host: {:?}", str::from_utf8(data).unwrap());
        self.host.extend_from_slice(data);
        true
    }

    fn on_url_path(&mut self, data: &[u8]) -> bool {
        println!("on_url_path: {:?}", str::from_utf8(data).unwrap());
        self.path.extend_from_slice(data);
        true
    }

    fn on_url_port(&mut self, data: u16) -> bool {
        println!("on_url_port: {}", data);
        self.port = data;
        true
    }

    fn on_url_query_string(&mut self, data: &[u8]) -> bool {
        println!("on_url_query_string: {:?}", str::from_utf8(data).unwrap());
        self.query_string.extend_from_slice(data);
        true
    }

    fn on_url_scheme(&mut self, data: &[u8]) -> bool {
        println!("on_url_scheme: {:?}", str::from_utf8(data).unwrap());
        self.scheme.extend_from_slice(data);
        true
    }
}

#[test]
fn parse_url_full() {
    let mut h = H{fragment: Vec::new(), host: Vec::new(), path: Vec::new(), port: 0,
                  query_string: Vec::new(), scheme: Vec::new()};

    assert!(match parse_url(&mut h, b"http://www.host.com:8080/just/a/path?query_string#fragment") {
        Ok(true) => true,
        _        => false
    });

    assert_eq!(h.scheme, b"http");
    assert_eq!(h.host, b"www.host.com");
    assert_eq!(h.port, 8080);
    assert_eq!(h.path, b"/just/a/path");
    assert_eq!(h.query_string, b"query_string");
    assert_eq!(h.fragment, b"fragment");
}

#[test]
fn parse_url_full_no_port() {
    let mut h = H{fragment: Vec::new(), host: Vec::new(), path: Vec::new(), port: 0,
                  query_string: Vec::new(), scheme: Vec::new()};

    assert!(match parse_url(&mut h, b"http://www.host.com/just/a/path?query_string#fragment") {
        Ok(true) => true,
        _        => false
    });

    assert_eq!(h.scheme, b"http");
    assert_eq!(h.host, b"www.host.com");
    assert_eq!(h.path, b"/just/a/path");
    assert_eq!(h.query_string, b"query_string");
    assert_eq!(h.fragment, b"fragment");
}

#[test]
fn parse_url_partial() {
    let mut h = H{fragment: Vec::new(), host: Vec::new(), path: Vec::new(), port: 0,
                  query_string: Vec::new(), scheme: Vec::new()};

    assert!(match parse_url(&mut h, b"/just/a/path?query_string#fragment") {
        Ok(true) => true,
        _        => false
    });

    assert_eq!(h.path, b"/just/a/path");
    assert_eq!(h.query_string, b"query_string");
    assert_eq!(h.fragment, b"fragment");
}

#[test]
fn parse_url_scheme_error() {
    let mut h = H{fragment: Vec::new(), host: Vec::new(), path: Vec::new(), port: 0,
                  query_string: Vec::new(), scheme: Vec::new()};

    assert!(match parse_url(&mut h, b"http:") {
        Err(UrlError::Scheme(_,_)) => true,
        _                          => false
    });
}

#[test]
fn parse_url_host_error() {
    let mut h = H{fragment: Vec::new(), host: Vec::new(), path: Vec::new(), port: 0,
                  query_string: Vec::new(), scheme: Vec::new()};

    assert!(match parse_url(&mut h, b"http://www.host\r.com") {
        Err(UrlError::Host(_,_)) => true,
        _                        => false
    });
}

#[test]
fn parse_url_port_error() {
    let mut h = H{fragment: Vec::new(), host: Vec::new(), path: Vec::new(), port: 0,
                  query_string: Vec::new(), scheme: Vec::new()};

    assert!(match parse_url(&mut h, b"http://www.host.com:99999") {
        Err(UrlError::Port(_,_)) => true,
        _                        => false
    });

    assert!(match parse_url(&mut h, b"http://www.host.com:80\r80") {
        Err(UrlError::Port(_,_)) => true,
        _                        => false
    });
}

#[test]
fn parse_url_path_error() {
    let mut h = H{fragment: Vec::new(), host: Vec::new(), path: Vec::new(), port: 0,
                  query_string: Vec::new(), scheme: Vec::new()};

    assert!(match parse_url(&mut h, b"http://www.host.com:80/just/a/path\r") {
        Err(UrlError::Path(_,_)) => true,
        _                        => false
    });
}

#[test]
fn parse_url_query_string_error() {
    let mut h = H{fragment: Vec::new(), host: Vec::new(), path: Vec::new(), port: 0,
                  query_string: Vec::new(), scheme: Vec::new()};

    assert!(match parse_url(&mut h, b"http://www.host.com:80/just/a/path?query\r_string") {
        Err(UrlError::QueryString(_,_)) => true,
        _                               => false
    });
}

#[test]
fn parse_url_fragment_error() {
    let mut h = H{fragment: Vec::new(), host: Vec::new(), path: Vec::new(), port: 0,
                  query_string: Vec::new(), scheme: Vec::new()};

    assert!(match parse_url(&mut h, b"http://www.host.com:80/just/a/path?query_string#frag\rment") {
        Err(UrlError::Fragment(_,_)) => true,
        _                            => false
    });
}