extern crate http_box;

use http_box::fsm::Success;
use http_box::http1::{ HttpHandler,
                       Parser,
                       State };

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::str;

struct HeadHandler;

impl HttpHandler for HeadHandler {
}

struct ChunkEncodedHandler {
    pub count:      usize,
    pub data:       Vec<u8>,
    pub extensions: Vec<u8>,
    pub length:     usize,
    pub name_buf:   Vec<u8>,
    pub state:      State,
    pub trailers:   HashMap<String, String>,
    pub value_buf:  Vec<u8>
}

impl ChunkEncodedHandler {
    fn clear(&mut self) {
        self.length = 0;

        self.data.clear();
        self.extensions.clear();
        self.trailers.clear();
    }

    fn flush_trailer(&mut self) {
        if self.name_buf.len() > 0 && self.value_buf.len() > 0 {
            self.trailers.insert(
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

impl HttpHandler for ChunkEncodedHandler {
    fn on_chunk_begin(&mut self) -> bool {
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

    fn on_chunk_data(&mut self, data: &[u8]) -> bool {
        self.data.extend_from_slice(data);

        true
    }

    fn on_chunk_extension_finished(&mut self) -> bool {
        true
    }

    fn on_chunk_extension_name(&mut self, name: &[u8]) -> bool {
        self.extensions.extend_from_slice(name);

        true
    }

    fn on_chunk_extension_value(&mut self, value: &[u8]) -> bool {
        self.extensions.extend_from_slice(value);

        true
    }

    fn on_chunk_extensions_finished(&mut self) -> bool {
        true
    }

    fn on_chunk_length(&mut self, length: usize) -> bool {
        self.length = length;

        true
    }

    fn on_header_name(&mut self, name: &[u8]) -> bool {
        if self.state == State::HeaderValue {
            self.flush_trailer();
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
        self.flush_trailer();

        true
    }
}

#[test]
fn chunk_encoded_body() {
    let mut d = Vec::new();

    File::open("tests/http1_data/chunk_encoded.dat").unwrap().read_to_end(&mut d);

    // parse head
    let mut s  = d.as_slice();
    let mut hp = Parser::new();

    match hp.resume(&mut HeadHandler, &s) {
        Ok(Success::Finished(length)) => {
            s = &s[length..];
        },
        _ => panic!()
    }

    // parse chunk encoded
    let mut cp = Parser::new();

    cp.init_chunked();

    let mut h = ChunkEncodedHandler{
        count:      0,
        data:       Vec::new(),
        extensions: Vec::new(),
        length:     0,
        name_buf:   Vec::new(),
        state:      State::None,
        trailers:   HashMap::new(),
        value_buf:  Vec::new()
    };

    // first chunk entry
    match cp.resume(&mut h, &s) {
        Ok(Success::Callback(length)) => {
            // adjust the slice since we've parsed one entry already
            s = &s[length..];
        },
        _ => panic!()
    }

    assert_eq!(
        h.trailers.len(),
        0
    );

    assert_eq!(
        h.length,
        23
    );

    assert_eq!(
        h.data,
        b"This is the first chunk"
    );

    // clear saved data
    h.clear();

    // second chunk entry
    match cp.resume(&mut h, &s) {
        Ok(Success::Callback(length)) => {
            // adjust the slice since we've parsed one entry already
            s = &s[length..];
        },
        _ => panic!()
    }

    assert_eq!(
        h.trailers.len(),
        0
    );

    assert_eq!(
        h.length,
        24
    );

    assert_eq!(
        h.data,
        b"This is the second chunk"
    );

    // clear saved data
    h.clear();

    // second chunk entry
    match cp.resume(&mut h, &s) {
        Ok(Success::Finished(_)) => {
        },
        _ => panic!()
    }

    assert_eq!(
        h.trailers.len(),
        2
    );

    assert_eq!(
        h.length,
        0
    );

    assert_eq!(
        h.data,
        b""
    );

    assert_eq!(
        h.trailers.get("trailer1").unwrap(),
        "This is trailer 1"
    );

    assert_eq!(
        h.trailers.get("trailer2").unwrap(),
        "This is trailer 2"
    );
}
