extern crate http_box;

use http_box::fsm::Success;
use http_box::http1::{ HttpHandler,
                       Parser,
                       State };
use http_box::util::FieldIterator;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::str;

struct HeadHandler {
    pub headers:   HashMap<String, String>,
    pub name_buf:  Vec<u8>,
    pub state:     State,
    pub value_buf: Vec<u8>
}

impl HeadHandler {
    pub fn new() -> HeadHandler {
        HeadHandler{
            headers:   HashMap::new(),
            name_buf:  Vec::new(),
            state:     State::None,
            value_buf: Vec::new()
        }
    }

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
}

struct MultipartHandler {
    pub count:     usize,
    pub data:      Vec<u8>,
    pub headers:   HashMap<String, String>,
    pub name_buf:  Vec<u8>,
    pub state:     State,
    pub value_buf: Vec<u8>
}

impl MultipartHandler {
    pub fn new() -> MultipartHandler {
        MultipartHandler{
            count:     0,
            data:      Vec::new(),
            headers:   HashMap::new(),
            name_buf:  Vec::new(),
            state:     State::None,
            value_buf: Vec::new()
        }
    }

    fn clear(&mut self) {
        self.data.clear();
        self.headers.clear();
    }

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

impl HttpHandler for MultipartHandler {
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

    fn on_multipart_begin(&mut self) -> bool {
        self.count += 1;

        if self.count > 1 {
            // we found a new piece of data, and it's not the first one, so force an exit
            // so we can compare
            false
        } else {
            // first piece of data, continue as normal
            true
        }
    }

    fn on_multipart_data(&mut self, data: &[u8]) -> bool {
        self.data.extend_from_slice(data);

        true
    }
}

#[test]
fn multipart() {
    let mut d = Vec::new();

    File::open("tests/http1_data/multipart.dat").unwrap().read_to_end(&mut d);

    let mut s  = d.as_slice();
    let mut hh = HeadHandler::new();
    let mut p  = Parser::new();

    // parse head
    match p.resume(&mut hh, &s) {
        Ok(Success::Finished(length)) => {
            // adjust the slice since we've parsed the head already
            s = &s[length..];
        },
        _ => panic!()
    }

    // get boundary
    let mut b = None;

    for (name, value) in FieldIterator::new(
        hh.headers.get("content-type").unwrap().as_bytes(),
        b';',
        true
    ) {
        if name == "boundary" {
            b = value;
        }
    }

    // parse multipart
    let mut mh = MultipartHandler::new();
    let mut p  = Parser::new();

    p.init_multipart();
    p.set_boundary(b.as_ref().unwrap().as_bytes());

    // first multipart entry
    match p.resume(&mut mh, &s) {
        Ok(Success::Callback(length)) => {
            // adjust the slice since we've parsed one entry already
            s = &s[length..];
        },
        _ => panic!()
    }

    assert_eq!(
        mh.headers.len(),
        1
    );

    assert_eq!(
        mh.headers.get("content-disposition").unwrap(),
        "form-data; name=\"first_name\""
    );

    assert_eq!(
        mh.data,
        b"Ada"
    );

    // clear saved data
    mh.clear();

    // second multipart entry
    match p.resume(&mut mh, &s) {
        Ok(Success::Callback(length)) => {
            // adjust the slice since we've parsed one entry already
            s = &s[length..];
        },
        _ => panic!()
    }

    assert_eq!(
        mh.headers.len(),
        1
    );

    assert_eq!(
        mh.headers.get("content-disposition").unwrap(),
        "form-data; name=\"last_name\""
    );

    assert_eq!(
        mh.data,
        b"Lovelace"
    );

    // clear saved data
    mh.clear();

    // third multipart entry
    match p.resume(&mut mh, &s) {
        Ok(Success::Callback(length)) => {
            // adjust the slice since we've parsed one entry already
            s = &s[length..];
        },
        _ => panic!()
    }

    assert_eq!(
        mh.headers.len(),
        2
    );

    assert_eq!(
        mh.headers.get("content-disposition").unwrap(),
        "form-data; name=\"file1\"; filename=\"rust-slide.jpg\""
    );

    assert_eq!(
        mh.headers.get("content-type").unwrap(),
        "image/jpeg"
    );

    assert_eq!(
        mh.data.len(),
        62260
    );

    // clear saved data
    mh.clear();

    // fourth multipart entry
    match p.resume(&mut mh, &s) {
        Ok(Success::Finished(_)) => {
        },
        _ => panic!()
    }

    assert_eq!(
        mh.headers.len(),
        2
    );

    assert_eq!(
        mh.headers.get("content-disposition").unwrap(),
        "form-data; name=\"file2\"; filename=\"rustacean.png\""
    );

    assert_eq!(
        mh.headers.get("content-type").unwrap(),
        "image/png"
    );

    assert_eq!(
        mh.data.len(),
        38310
    );
}
