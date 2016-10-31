use http_box::http1::{HttpHandler, Parser};

pub struct Handler {
    pub method: Vec<u8>,
    pub status: Vec<u8>,
    pub status_code: u16,
    pub status_finished: bool,
    pub url: Vec<u8>,
    pub version_major: u16,
    pub version_minor: u16
}

impl HttpHandler for Handler {
    pub fn is_request(&self) -> bool {
        self.method.len() > 0
    }

    pub fn is_status_finished(&self) -> bool {
        self.status_finished
    }

    pub fn on_method(&mut self, data: &[u8]) -> bool {
        self.method.extend_from_slice(data);
        true
    }

    pub fn on_status(&mut self, data: &[u8]) -> bool {
        self.status.extend_from_slice(data);
        true
    }

    pub fn on_status_code(&mut self, code: u16) -> bool {
        self.status_code = code;
        true
    }

    pub fn on_status_finished(&mut self) -> bool {
        self.status_finished = true;
        true
    }

    pub fn on_url(&mut self, data: &[u8]) -> bool {
        self.url.extend_from_slice(data);
        true
    }

    pub fn on_version(&mut self, major: u16, minor: u16) -> bool {
        self.version_major = major;
        self.version_minor = minor;
        true
    }
}

#[test]
fn main() {
    // init handler and parser
    let mut h = Handler{ method: Vec::new(),
                         status: Vec::new(),
                         status_code: 0,
                         status_finished: false,
                         url: Vec::new(),
                         version_major: 0,
                         version_minor: 0 };

    let mut p = Parser::new();

    // parse request
    p.parse_head(&mut h, b"GET /url HTTP/1.0\r\n");

    assert_eq!(true, h.is_status_finished());
    assert_eq!(true, h.is_request());
    assert_eq!(h.method, b"GET");
    assert_eq!(h.url, b"/url");
}
