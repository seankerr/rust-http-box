use http_box::http1::{HttpHandler, Parser};
use std::collections::HashMap;

pub struct Handler {
    pub headers: HashMap<Vec<u8>,Vec<u8>>,
    pub field: Vec<u8>,
    pub value: Vec<u8>
}

impl HttpHandler for Handler {
    fn flush_header(&mut self) {
        if self.field.len() > 0 && self.value.len() > 0 {
            self.headers.insert(self.field.clone(), self.value.clone());
        }

        self.field.clear();
        self.value.clear();
    }

    pub fn on_header_field(&mut self, data: &[u8]) -> bool {
        if self.value.len() > 0 {
            self.flush_header();
        }

        self.field.extend_from_slice(data);
        true
    }

    pub fn on_header_value(&mut self, data: &[u8]) -> bool {
        self.value.extend_from_slice(data);
        true
    }

    pub fn on_headers_finished(&mut self) -> bool {
        self.flush_header();
        true
    }
}

#[test]
fn main() {
    // init handler and parser
    let mut h = Handler{ headers: HashMap::new(),
                         field: Vec::new(),
                         value: Vec::new() };

    let mut p = Parser::new();

    // parse headers
    p.parse_head(&mut h, b"GET /url HTTP/1.0\r\n\
                           Header1: Value 1\r\n\
                           Header2: Value 2\r\n\r\n");

    assert_eq!(true, h.is_status_finished());
    assert_eq!(true, h.is_request());
    assert_eq!(h.method, b"GET");
    assert_eq!(h.url, b"/url");
}
