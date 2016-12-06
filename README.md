# rust-http-box

![Build: Passing](https://img.shields.io/badge/build-passing-brightgreen.svg)
![dev: 0.1.0](https://img.shields.io/badge/dev-0.1.0-ff69b4.svg)
![license: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)

## What is http-box?

http-box is a push oriented HTTP parser with the goal of remaining fast and bare bones. There are
several HTTP client and server libraries available, but http-box attempts to stay as far away from
abstract as possible, giving the developer absolute and full control over how HTTP data is
processed.

## Features

- Push oriented and will process a single byte at a time
- Callback oriented with the ability to break out of the parser loop
- Headers are normalized to lower-case
- Error handling is a breeze
- Parse HTTP phases separately:
  - Head
    - Request / Response
    - Headers
  - Body
    - Multipart
      - Headers
      - Data
    - Chunk Transfer-Encoded
      - Chunk Length
      - Extensions
      - Chunk Data
      - Trailers
    - URL encoded
      - Parameters
- Zero copy philosophy
- DoS protection is easily supported
- Fast!
- Use with any networking library

## API Documentation

https://docs.rs/http-box/0.1.0/http_box/

## Quick Docs

### Parser

[Parser](http://docs.rs/http-box/0.1.0/http_box/http1/struct.Parser.html) is the guts of
the library. It provides only necessary components for parsing HTTP data.

### HttpHandler

Implementing [HttpHandler](http://docs.rs/http-box/0.1.0/http_box/http1/trait.HttpHandler.html)
is how you provide a custom callback implementation. It is optional to provide multiple
implementations based on which type of data is being parsed: head, chunked transfer-encoded,
multipart, URL encoded, etc. It is also suggested since it lends itself to clarity.

### Callbacks

In a typical application, callbacks receive arguments that are complete pieces of data. However,
[Parser](http://docs.rs/http-box/0.1.0/http_box/http1/struct.Parser.html) parses data, and
because of this, it must operate one byte at a type. Moreoever, the data being parsed is often
coming from a network connection, and is received as incomplete pieces of data. To stick to the
zero-copy philosophy, and to avoid buffering, callbacks are executed as frequent as necessary.

### Tracking State

Sometimes multiple states need to work together to produce a single result. A good example of this
is when headers are being parsed. The callback for the header name may be called multiple times in
order to receive the full header name. And the same is true for the header value. It isn't until the
header value is complete, that the header name/value pair can be stored.

This is where the [State](http://docs.rs/http-box/0.1.0/http_box/http1/enum.State.html) enum
comes into play. You can use this to track the current state when a callback is executed. There is
nothing mysterious about this enum. It's a helper type with the objective of simplifying state
tracking.

### Example

```rust
extern crate http_box;

use http_box::http1::{HttpHandler, Parser, State};
use std::collections::HashMap;

// provide a container for the data we wish to track
pub struct Handler {
    pub headers: HashMap<String,String>,
    pub initial_finished: bool,
    pub method: Vec<u8>,
    pub name: Vec<u8>,
    pub state: State,
    pub status: Vec<u8>,
    pub status_code: u16,
    pub url: Vec<u8>,
    pub value: Vec<u8>,
    pub version_major: u16,
    pub version_minor: u16
}

impl Handler {
    fn flush_header(&mut self) {
        if self.name.len() > 0 && self.value.len() > 0 {
            self.headers.insert(String::from_utf8(self.name.clone()).unwrap(),
                                String::from_utf8(self.value.clone()).unwrap());
        }

        self.name.clear();
        self.value.clear();
    }

    pub fn is_request(&self) -> bool {
        self.method.len() > 0
    }

    pub fn is_initial_finished(&self) -> bool {
        self.initial_finished
    }
}

// implement the callbacks we are listening for
impl HttpHandler for Handler {
    // receive a slice of header name
    fn on_header_name(&mut self, data: &[u8]) -> bool {
        if self.state == State::HeaderValue {
            self.flush_header();

            self.state = State::HeaderName;
        }

        self.name.extend_from_slice(data);
        true
    }

    // receive a slice of header value
    fn on_header_value(&mut self, data: &[u8]) -> bool {
        self.state = State::HeaderValue;
        self.value.extend_from_slice(data);
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
    // init callback handler
    let mut p = Parser::new_head(
                    Handler{ headers:          HashMap::new(),
                             initial_finished: false,
                             method:           Vec::new(),
                             name:             Vec::new(),
                             state:            State::None,
                             status:           Vec::new(),
                             status_code:      0,
                             url:              Vec::new(),
                             value:            Vec::new(),
                             version_major:    0,
                             version_minor:    0 }
                );

    // parse some head data
    p.resume(b"GET /url?query HTTP/1.0\r\n\
               Header1: This is the first header\r\n\
               Header2: This is the second header\r\n\
               \r\n");

    // compare our data
    let h = p.handler();

    assert!(h.is_initial_finished());
    assert!(h.is_request());
    assert_eq!(h.method, b"GET");
    assert_eq!(h.url, b"/url?query");
    assert_eq!(h.headers.get("header1").unwrap(), "This is the first header");
    assert_eq!(h.headers.get("header2").unwrap(), "This is the second header");
}
```
