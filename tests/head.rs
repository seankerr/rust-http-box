extern crate http_box;

use http_box::http1::{ HttpHandler,
                       Parser,
                       State };

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

struct HeadHandler {
    pub headers:       HashMap<String, String>,
    pub method:        Vec<u8>,
    pub name_buf:      Vec<u8>,
    pub state:         State,
    pub url:           Vec<u8>,
    pub value_buf:     Vec<u8>,
    pub version_major: u16,
    pub version_minor: u16
}

impl HeadHandler {
    fn flush_header(&mut self) {
        if self.name_buf.len() > 0 && self.value_buf.len() > 0 {
            self.headers.insert(
                unsafe {
                    let mut s = String::with_capacity(self.name_buf.len());

                    s.as_mut_vec().extend_from_slice(&self.name_buf);
                    s
                },
                unsafe {
                    let mut s = String::with_capacity(self.value_buf.len());

                    s.as_mut_vec().extend_from_slice(&self.value_buf);
                    s
                }
            );
        }

        self.name_buf.clear();
        self.value_buf.clear();
    }
}

impl HttpHandler for HeadHandler {
    fn on_header_name(&mut self, name: &[u8]) -> bool {
        if self.state == State::HeaderValue {
            self.flush_header();
        }

        self.name_buf.extend_from_slice(name);

        self.state = State::HeaderName;
        true
    }

    fn on_header_value(&mut self, value: &[u8]) -> bool {
        self.value_buf.extend_from_slice(value);

        self.state = State::HeaderValue;
        true
    }

    fn on_headers_finished(&mut self) -> bool {
        self.flush_header();

        true
    }

    fn on_method(&mut self, method: &[u8]) -> bool {
        self.method.extend_from_slice(method);

        true
    }

    fn on_url(&mut self, url: &[u8]) -> bool {
        self.url.extend_from_slice(url);

        true
    }

    fn on_version(&mut self, major: u16, minor: u16) -> bool {
        self.version_major = major;
        self.version_minor = minor;

        true
    }
}

#[test]
fn head() {
    let mut d = Vec::new();

    File::open("tests/data/multipart.dat").unwrap().read_to_end(&mut d);

    let mut h = HeadHandler{ headers:       HashMap::new(),
                             method:        Vec::new(),
                             name_buf:      Vec::new(),
                             state:         State::None,
                             url:           Vec::new(),
                             value_buf:     Vec::new(),
                             version_major: 0,
                             version_minor: 0 };
    let mut p = Parser::new();

    p.init_head();
    p.resume(&mut h, &d);

    assert_eq!(h.method,
               b"POST");

    assert_eq!(h.url,
               b"/multipart");

    assert_eq!(h.version_major,
               1);

    assert_eq!(h.version_minor,
               1);

    assert_eq!(h.headers.get("accept").unwrap(),
               "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8");

    assert_eq!(h.headers.get("accept-encoding").unwrap(),
               "gzip, deflate, br");

    assert_eq!(h.headers.get("accept-language").unwrap(),
               "en-US,en;q=0.8");

    assert_eq!(h.headers.get("cache-control").unwrap(),
               "max-age=0");

    assert_eq!(h.headers.get("connection").unwrap(),
               "keep-alive");

    assert_eq!(h.headers.get("content-length").unwrap(),
               "101106");

    assert_eq!(h.headers.get("content-type").unwrap(),
               "multipart/form-data; boundary=----WebKitFormBoundaryPplB3C4KqDmwKzm4");

    assert_eq!(h.headers.get("host").unwrap(),
               "localhost");

    assert_eq!(h.headers.get("origin").unwrap(),
               "null");

    assert_eq!(h.headers.get("user-agent").unwrap(),
               "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/54.0.2840.87 Safari/537.36");

    assert_eq!(h.headers.get("upgrade-insecure-requests").unwrap(),
               "1");
}
