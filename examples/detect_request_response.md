# Detecting Request or Response

[HttpHandler](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html) has 6 callback
functions related to the status line:

**Request**

- [on_method()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_method): Receive method details
- [on_url()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_url): Receive URL details

**Response**

- [on_status()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_status): Receive status details
- [on_status_code()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_status_code): Receive status code details

**Request and Response**
- [on_status_finished()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_status_finished): Indicates that the status line has finished parsing successfully
- [on_version()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_version): Receive HTTP major and minor version

## Example

Once the first line of a request or response is processed, the `on_status_finished()` callback
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
    pub method: Vec<u8>,
    pub status: Vec<u8>,
    pub status_code: u16,
    pub status_finished: bool,
    pub url: Vec<u8>,
    pub version_major: u16,
    pub version_minor: u16
}

impl Handler {
    pub fn is_request(&self) -> bool {
        self.method.len() > 0
    }

    pub fn is_status_finished(&self) -> bool {
        self.status_finished
    }
}

impl HttpHandler for Handler {
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

    fn on_status_finished(&mut self) -> bool {
        self.status_finished = true;
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
```
