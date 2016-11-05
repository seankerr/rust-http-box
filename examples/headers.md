# Headers

[HttpHandler](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html) has 3 callback functions that are related to headers:

- [on_header_name()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_header_name): Receive header name details
- [on_header_value()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_header_value): Receive header value details
- [on_headers_finished()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_headers_finished): Indicates headers have finished parsing

Similar to status line parsing, when the headers are finished parsing, the `on_headers_finished()`
callback will be executed. You may be wondering how to detect when a new header name or value
is received, and how to keep it separate from the recent header name and value. This can be
achieved with a bit of finesse.

You will notice that the header name used to retrieve the header from the `HashMap` is lower-cased.
This is because header names are normalized to lower-case automatically.

## Example

```rust
extern crate http_box;

use http_box::http1::{HttpHandler, Parser, State};
use std::collections::HashMap;

pub struct Handler {
    pub headers: HashMap<String,String>,
    pub state: State,
    pub name: Vec<u8>,
    pub value: Vec<u8>
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
}

impl HttpHandler for Handler {
    fn on_header_name(&mut self, data: &[u8]) -> bool {
        if self.state == State::HeaderValue {
            self.flush_header();

            self.state = State::HeaderName;
        }

        self.name.extend_from_slice(data);
        true
    }

    fn on_header_value(&mut self, data: &[u8]) -> bool {
        self.state = State::HeaderValue;
        self.value.extend_from_slice(data);
        true
    }

    fn on_headers_finished(&mut self) -> bool {
        self.flush_header();
        true
    }
}

fn main() {
    // init handler and parser
    let mut h = Handler{ headers: HashMap::new(),
                         state: State::None,
                         name: Vec::new(),
                         value: Vec::new() };

    let mut p = Parser::new();

    // parse headers
    p.parse_head(&mut h, b"GET /url HTTP/1.0\r\n\
                           Header1: Value 1\r\n\
                           Header2: Value 2\r\n\r\n");

    assert_eq!("Value 1", h.headers.get("header1").unwrap());
    assert_eq!("Value 2", h.headers.get("header2").unwrap());
}
```
