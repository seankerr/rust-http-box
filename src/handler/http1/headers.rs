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
///
/// # Request Example
///
/// ```
/// use http_box::HeadersHttp1Handler;
/// use http_box::http1::Parser;
///
/// let mut h = HeadersHttp1Handler::new();
/// let mut p = Parser::new();
///
/// p.parse_headers(&mut h, b"GET / HTTP/1.1\r\nHeader: value\r\n\r\n", 1000);
///
/// // header fields are normalized to lower-case
/// assert_eq!("value", h.get_headers().get("header").unwrap());
///
/// // request details
/// assert_eq!("GET", h.get_method());
/// assert_eq!("/", h.get_url());
/// assert_eq!(1, h.get_version_major());
/// assert_eq!(1, h.get_version_minor());
/// ```
///
/// # Response Example
///
/// ```
/// use http_box::HeadersHttp1Handler;
/// use http_box::http1::Parser;
///
/// let mut h = HeadersHttp1Handler::new();
/// let mut p = Parser::new();
///
/// p.parse_headers(&mut h, b"HTTP/1.1 200 OK\r\nHeader: value\r\n\r\n", 1000);
///
/// // header fields are normalized to lower-case
/// //assert_eq!("value", h.get_headers().get("header").unwrap());
///
/// // response details
/// //assert_eq!(1, h.get_version_major());
/// //assert_eq!(1, h.get_version_minor());
/// //assert_eq!(200, h.get_status_code());
/// //assert_eq!("OK", h.get_status());
/// ```
pub struct HeadersHttp1Handler {
    /// Header field buffer.
    header_field: String,

    /// Header field/value toggle.
    header_toggle: bool,

    /// Header value buffer.
    header_value: String,

    /// Map of all headers.
    headers: HashMap<String,String>,

    /// Request method.
    method: String,

    /// Response status.
    status: String,

    /// Response status code.
    status_code: u16,

    /// Request URL.
    url: String,

    /// HTTP major version.
    version_major: u16,

    /// HTTP minor version.
    version_minor: u16
}

impl HeadersHttp1Handler {
    /// Create a new `HeadersHttp1Handler`.
    pub fn new() -> HeadersHttp1Handler {
        HeadersHttp1Handler {
            header_field:  String::new(),
            header_toggle: false,
            header_value:  String::new(),
            headers:       HashMap::new(),
            method:        String::new(),
            status:        String::new(),
            status_code:   0,
            url:           String::new(),
            version_major: 0,
            version_minor: 0
        }
    }

    /// Flush the most recent header field/value.
    fn flush(&mut self) {
        if self.header_field.len() > 0 {
            self.headers.insert(self.header_field.clone(), self.header_value.clone());
        }

        self.header_field.clear();
        self.header_value.clear();
    }

    /// Retrieve the headers.
    pub fn get_headers(&self) -> &HashMap<String,String> {
        &self.headers
    }

    /// Retrieve the request method.
    pub fn get_method(&self) -> &str {
        &self.method
    }

    /// Retrieve the response status.
    pub fn get_status(&self) -> &str {
        &self.status
    }

    /// Retrieve the response status code.
    pub fn get_status_code(&self) -> u16 {
        self.status_code
    }

    /// Retrieve the request URL.
    pub fn get_url(&self) -> &str {
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

        unsafe {
            self.header_field
                .as_mut_vec()
                .extend_from_slice(field);
        }

        true
    }

    fn on_header_value(&mut self, value: &[u8]) -> bool {
        unsafe {
            self.header_value
                .as_mut_vec()
                .extend_from_slice(value);
        }

        self.header_toggle = true;
        true
    }

    fn on_headers_finished(&mut self) -> bool {
        self.flush();
        true
    }

    fn on_method(&mut self, method: &[u8]) -> bool {
        unsafe {
            self.method
                .as_mut_vec()
                .extend_from_slice(method);
        }

        true
    }

    fn on_status(&mut self, status: &[u8]) -> bool {
        unsafe {
            self.status
                .as_mut_vec()
                .extend_from_slice(status);
        }

        true
    }

    fn on_status_code(&mut self, code: u16) -> bool {
        self.status_code = code;
        true
    }

    fn on_url(&mut self, url: &[u8]) -> bool {
        unsafe {
            self.url
                .as_mut_vec()
                .extend_from_slice(url);
        }

        true
    }

    fn on_version_major(&mut self, major: u8) -> bool {
        self.version_major = major;
        true
    }

    fn on_version_minor(&mut self, minor: u8) -> bool {
        self.version_minor = minor;
        true
    }
}
