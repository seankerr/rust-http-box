# Detecting Request or Response

Once the first line of a request or response is processed, the callback *on_status_finished()*
will be executed. At this point it will be possible to detect whether or not the HTTP type is
a request or response by checking the callback data that was stored.

For all intents and purposes, the only data we need to store in order to determine the HTTP type
is a boolean. We could simply set the boolean value to true if the `on_method()` callback was
executed. However, here is a more verbose example that stores all status details.

```rust
use http_box::http1::{HttpHandler, Parser};

struct Handler {
    method: Vec<u8>,
    status: Vec<u8>,
    status_code: u16,
    url: Vec<u8>,
    version_major: u16,
    version_minor: u16
}

impl HttpHandler for Handler {
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
        if self.method.len() > 0 {
            // this is a request
        } else {
            // this is a response
        }
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
```
