// +-----------------------------------------------------------------------------------------------+
// | Copyright 2016 Sean Kerr                                                                      |
// |                                                                                               |
// | Licensed under the Apache License, Version 2.0 (the "License");                               |
// | you may not use this file except in compliance with the License.                              |
// | You may obtain a copy of the License at                                                       |
// |                                                                                               |
// |  http://www.apache.org/licenses/LICENSE-2.0                                                   |
// |                                                                                               |
// | Unless required by applicable law or agreed to in writing, software                           |
// | distributed under the License is distributed on an "AS IS" BASIS,                             |
// | WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.                      |
// | See the License for the specific language governing permissions and                           |
// | limitations under the License.                                                                |
// +-----------------------------------------------------------------------------------------------+
// | Author: Sean Kerr <sean@code-box.org>                                                         |
// +-----------------------------------------------------------------------------------------------+

//! [`Http1Handler`](../../../http1/trait.Http1Handler.html) implementation for processing the
//! initial request/response line, and headers.

use http1::Http1Handler;

use std::collections::HashMap;

/// `HeadersHttp1Handler` is a suitable handler for the following parser functions:
///
/// - [`Parser::parse_headers()`](../../../http1/struct.Parser.html#method.parse_headers)
pub struct HeadersHttp1Handler {
    header_field:  Vec<u8>,
    header_toggle: bool,
    header_value:  Vec<u8>,
    headers:       HashMap<Vec<u8>,Vec<u8>>,
    method:        Vec<u8>,
    status:        Vec<u8>,
    status_code:   u16,
    url:           Vec<u8>,
    version_major: u16,
    version_minor: u16
}

impl HeadersHttp1Handler {
    /// Create a new `HeadersHttp1Handler`.
    pub fn new() -> HeadersHttp1Handler {
        HeadersHttp1Handler {
            header_field:  Vec::new(),
            header_toggle: false,
            header_value:  Vec::new(),
            headers:       HashMap::new(),
            method:        Vec::new(),
            status:        Vec::new(),
            status_code:   0,
            url:           Vec::new(),
            version_major: 0,
            version_minor: 0
        }
    }

    /// Flush the most recent header field/value.
    fn flush(&mut self) {
        if self.header_field.len() > 0 {
            let mut field = Vec::with_capacity(self.header_field.len());
            let mut value = Vec::with_capacity(self.header_value.len());

            field.extend_from_slice(&self.header_field);
            value.extend_from_slice(&self.header_value);

            self.headers.insert(field, value);
        }

        self.header_field.clear();
        self.header_value.clear();
    }

    /// Retrieve the headers.
    pub fn get_headers(&self) -> &HashMap<Vec<u8>,Vec<u8>> {
        &self.headers
    }

    /// Retrieve the request method.
    pub fn get_method(&self) -> &[u8] {
        &self.method
    }

    /// Retrieve the response status.
    pub fn get_status(&self) -> &[u8] {
        &self.status
    }

    /// Retrieve the response status code.
    pub fn get_status_code(&self) -> u16 {
        self.status_code
    }

    /// Retrieve the request URL.
    pub fn get_url(&self) -> &[u8] {
        &self.url
    }

    /// Retrieve the HTTP major version.
    pub fn get_version_major(&self) -> u16 {
        self.version_major
    }

    /// Retrieve the HTTP minor version.
    pub fn get_version_minor(&self) -> u16 {
        self.version_minor
    }

    /// Indicates that the parsed data is an HTTP request.
    pub fn is_request(&self) -> bool {
        self.method.len() > 0
    }

    /// Reset the handler back to its original state.
    pub fn reset(&mut self) {
        self.header_toggle = false;
        self.status_code   = 0;
        self.version_major = 0;
        self.version_minor = 0;

        self.header_field.clear();
        self.header_value.clear();
        self.headers.clear();
        self.method.clear();
        self.status.clear();
        self.url.clear();
    }
}

impl Http1Handler for HeadersHttp1Handler {
    fn on_header_field(&mut self, field: &[u8]) -> bool {
        if self.header_toggle {
            self.flush();

            self.header_toggle = false;
        }

        self.header_field.extend_from_slice(field);
        true
    }

    fn on_header_value(&mut self, value: &[u8]) -> bool {
        self.header_value.extend_from_slice(value);

        self.header_toggle = true;
        true
    }

    fn on_headers_finished(&mut self) -> bool {
        self.flush();
        true
    }

    fn on_method(&mut self, method: &[u8]) -> bool {
        self.method.extend_from_slice(method);
        true
    }

    fn on_status(&mut self, status: &[u8]) -> bool {
        self.status.extend_from_slice(status);
        true
    }

    fn on_status_code(&mut self, code: u16) -> bool {
        self.status_code = code;
        true
    }

    fn on_url(&mut self, url: &[u8]) -> bool {
        self.url.extend_from_slice(url);
        true
    }

    fn on_version(&mut self, major: u16, minor: u16) -> bool {
        self.version_major = major;
        self.version_minor = minor;
        true
    }
}
