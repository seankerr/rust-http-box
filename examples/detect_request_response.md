# Detecting Request or Response

[HttpHandler](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html) has 6 callback
functions related to the status line:

**Request**

- [on_method()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_method): Receive method
- [on_url()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_url): Receive URL
- [on_version()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_version): Receive HTTP major and minor version
- [on_initial_finished()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_initial_finished): Indicates status line parsing has finished

**Response**

- [on_version()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_version): Receive HTTP major and minor version
- [on_status()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_status): Receive status
- [on_status_code()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_status_code): Receive status code
- [on_initial_finished()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_initial_finished): Indicates status line parsing has finished

## Example

Once the first line of a request or response is processed, the `on_initial_finished()` callback
will be executed. At this point it will be possible to detect whether or not the HTTP type is
a request or response by checking the callback data that was stored.

For all intents and purposes, the only data we need to store in order to determine the HTTP type
is a boolean. However, the safest route to detection is to store a boolean that indicates status
line parsing has finished. We can provide a function that allows us to check the boolean, and then
another method to check the request method length.

```rust
extern crate http_box;

use http_box::http1::{HttpHandler, Parser};

pub struct Handler {
    pub initial_finished: bool,
    pub method: Vec<u8>,
    pub status: Vec<u8>,
    pub status_code: u16,
    pub url: Vec<u8>,
    pub version_major: u16,
    pub version_minor: u16
}

impl Handler {
    pub fn is_request(&self) -> bool {
        self.method.len() > 0
    }

    pub fn is_initial_finished(&self) -> bool {
        self.initial_finished
    }
}

impl HttpHandler for Handler {
    fn on_initial_finished(&mut self) -> bool {
        self.initial_finished = true;
        true
    }

    fn on_method(&mut self, data: &[u8]) -> bool {
        self.method.extend_from_slice(data);
        true
    }

    fn on_status(&mut self, data: &[u8]) -> bool {
        self.status.extend_from_slice(data);
        true
    }

    fn on_status_code(&mut self, code: u16) -> bool {
        self.status_code = code;
        true
    }

    fn on_url(&mut self, data: &[u8]) -> bool {
        self.url.extend_from_slice(data);
        true
    }

    fn on_version(&mut self, major: u16, minor: u16) -> bool {
        self.version_major = major;
        self.version_minor = minor;
        true
    }
}

fn main() {
    // init handler and parser
    let mut h = Handler{ initial_finished: false,
                         method: Vec::new(),
                         status: Vec::new(),
                         status_code: 0,
                         url: Vec::new(),
                         version_major: 0,
                         version_minor: 0 };

    let mut p = Parser::new();

    p.init_head();
    p.resume(&mut h, b"GET /url HTTP/1.0\r\n");

    assert_eq!(true, h.is_initial_finished());
    assert_eq!(true, h.is_request());
    assert_eq!(h.method, b"GET");
    assert_eq!(h.url, b"/url");
}
```
