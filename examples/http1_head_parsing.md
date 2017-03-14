# Head Parsing

When parsing HTTP/1.x content, everything prior to the body is considered the
head or headers. It starts off with the request or response line followed by
optional headers, and is terminated with a `\r\n\r\n` sequence.

Here is a sample of request head content:

```
GET /resource HTTP/1.1\r\n
Host: github.com\r\n
Connection: keep-alive\r\n
Accept-Encoding: gzip, deflate, sdch, br\r\n
Accept-Language: en-US,en;q=0.8\r\n
\r\n
```

Response head content format is identical, with the exception of the initial
line, which denotes the HTTP response details:

```
HTTP/1.1 200 OK\r\n
```

The headers are formatted the same for a request or response, however, the types
of headers being used will vary.

# Callbacks

[http_box::http1::HttpHandler](https://docs.rs/http-box/0.1.3/http_box/http1/trait.HttpHandler.html)
outlines the necessary callback functions we must implement in order to handle
parsed head data:

**Request:**

- `HttpHandler::on_method()`
- `HttpHandler::on_url()`

**Response:**

- `HttpHandler::on_status()`
- `HttpHandler::on_status_code()`

**Request + Response:**

- `HttpHandler::on_header_name()`
- `HttpHandler::on_header_value()`
- `HttpHandler::on_version()`

# Example

```rust
extern crate http_box;

use http_box::http1::{ HttpHandler, Parser, State };
use std::collections::HashMap;

// container for storing the parsed data
pub struct Handler {
    // headers
    pub headers: HashMap<String,String>,

    // indicates that head parsing has finished
    pub initial_finished: bool,

    // request method
    pub method: Vec<u8>,

    // buffer for accumulating header name
    pub header_name: Vec<u8>,

    // buffer for accumulating header value
    pub header_value: Vec<u8>,

    // current state
    pub state: State,

    // response status
    pub status: Vec<u8>,

    // response status code
    pub status_code: u16,

    // request URL
    pub url: Vec<u8>,

    // request + response major version
    pub version_major: u16,

    // request + response minor version
    pub version_minor: u16
}

// container implementation
impl Handler {
    pub fn new() -> Handler {
        Handler{
            headers:          HashMap::new(),
            initial_finished: false,
            method:           Vec::new(),
            header_name:      Vec::new(),
            header_value:     Vec::new(),
            state:            State::None,
            status:           Vec::new(),
            status_code:      0,
            url:              Vec::new(),
            version_major:    0,
            version_minor:    0
        }
    }

    fn flush_header(&mut self) {
        if self.header_name.len() > 0 && self.header_value.len() > 0 {
            self.headers.insert(
                String::from_utf8(self.header_name.clone()).unwrap(),
                String::from_utf8(self.header_value.clone()).unwrap()
            );
        }

        self.header_name.clear();
        self.header_value.clear();
    }

    pub fn is_request(&self) -> bool {
        self.method.len() > 0
    }

    pub fn is_initial_finished(&self) -> bool {
        self.initial_finished
    }
}

// our callback implementation
impl HttpHandler for Handler {
    // receive a slice of header name
    fn on_header_name(&mut self, data: &[u8]) -> bool {
        if self.state == State::HeaderValue {
            self.flush_header();

            self.state = State::HeaderName;
        }

        self.header_name.extend_from_slice(data);
        true
    }

    // receive a slice of header value
    fn on_header_value(&mut self, data: &[u8]) -> bool {
        self.state = State::HeaderValue;
        self.header_value.extend_from_slice(data);
        true
    }

    // executed when headers are finished
    fn on_headers_finished(&mut self) -> bool {
        self.flush_header();
        true
    }

    // executed when the initial request/response line is finished
    fn on_initial_finished(&mut self) -> bool {
        self.initial_finished = true;
        true
    }

    // receive a slice of request method
    fn on_method(&mut self, data: &[u8]) -> bool {
        self.method.extend_from_slice(data);
        true
    }

    // receive a slice of response status
    fn on_status(&mut self, data: &[u8]) -> bool {
        self.status.extend_from_slice(data);
        true
    }

    // receive response status code
    fn on_status_code(&mut self, code: u16) -> bool {
        self.status_code = code;
        true
    }

    // receive a slice of request url
    fn on_url(&mut self, data: &[u8]) -> bool {
        self.url.extend_from_slice(data);
        true
    }

    // receive request/response version
    fn on_version(&mut self, major: u16, minor: u16) -> bool {
        self.version_major = major;
        self.version_minor = minor;
        true
    }
}

fn main() {
    // init callback handler and parser
    let mut h = Handler::new();
    let mut p = Parser::new();

    // parse head content
    p.resume(
        &mut h,
        b"GET /resource?query HTTP/1.1\r\n\
          Host: github.com\r\n\
          Connection: close\r\n\
          Accept-Encoding: gzip, deflate, sdch, br\r\n\
          Accept-Language: en-US,en;q=0.8\r\n\
          \r\n"
    );

    assert!(h.is_initial_finished());
    assert!(h.is_request());

    assert_eq!(
        h.method,
        b"GET"
    );

    assert_eq!(
        h.url,
        b"/resource?query"
    );

    assert_eq!(
        h.headers.get("host").unwrap(),
        "github.com"
    );

    assert_eq!(
        h.headers.get("connection").unwrap(),
        "close"
    );

    assert_eq!(
        h.headers.get("accept-encoding").unwrap(),
        "gzip, deflate, sdch, br"
    );

    assert_eq!(
        h.headers.get("accept-language").unwrap(),
        "en-US,en;q=0.8"
    );
}
```
