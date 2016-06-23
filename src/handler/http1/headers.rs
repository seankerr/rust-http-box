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
/// - [`Parser::parse_headers()`](../http1/struct.Parser.html#method.parse_headers)
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
/// p.parse_headers(&mut h,
///                 b"GET / HTTP/1.1\r\n\
///                   Header1: value1\r\n\
///                   Header2: value2\r\n\
///                   \r\n\r\n", 0);
///
/// // header fields are normalized to lower-case
/// assert_eq!("value1", h.get_headers().get("header1").unwrap());
/// assert_eq!("value2", h.get_headers().get("header2").unwrap());
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
/// p.parse_headers(&mut h,
///                 b"HTTP/1.1 200 OK\r\n\
///                   Header1: value1\r\n\
///                   Header2: value2\r\n\
///                   \r\n\r\n", 0);
///
/// // header fields are normalized to lower-case
/// assert_eq!("value1", h.get_headers().get("header1").unwrap());
/// assert_eq!("value2", h.get_headers().get("header2").unwrap());
///
/// // response details
/// assert_eq!(1, h.get_version_major());
/// assert_eq!(1, h.get_version_minor());
/// assert_eq!(200, h.get_status_code());
/// assert_eq!("OK", h.get_status());
/// ```
pub struct HeadersHttp1Handler {
    /// Cookies.
    cookies: HashMap<String,String>,

    /// Header field buffer.
    field_buffer: String,

    /// Indicates that headers are finished parsing.
    finished: bool,

    /// Headers.
    headers: HashMap<String,String>,

    /// Request method.
    method: String,

    /// Response status.
    status: String,

    /// Response status code.
    status_code: u16,

    /// Header field/value toggle.
    toggle: bool,

    /// Request URL.
    url: String,

    /// Header value buffer.
    value_buffer: String,

    /// HTTP major version.
    version_major: u16,

    /// HTTP minor version.
    version_minor: u16
}

impl HeadersHttp1Handler {
    /// Create a new `HeadersHttp1Handler`.
    pub fn new() -> HeadersHttp1Handler {
        HeadersHttp1Handler {
            cookies:       HashMap::new(),
            field_buffer:  String::new(),
            finished:      false,
            headers:       HashMap::new(),
            method:        String::new(),
            status:        String::new(),
            status_code:   0,
            toggle:        false,
            url:           String::new(),
            value_buffer:  String::new(),
            version_major: 0,
            version_minor: 0
        }
    }

    /// Flush the most recent header field/value.
    fn flush(&mut self) {
        self.headers.insert(self.field_buffer.clone(), self.value_buffer.clone());

        self.field_buffer.clear();
        self.value_buffer.clear();
    }

    /// Retrieve the cookies.
    pub fn get_cookies(&self) -> &HashMap<String,String> {
        &self.cookies
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

    /// Indicates that the headers are finished parsing.
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Indicates that the parsed data is an HTTP request.
    pub fn is_request(&self) -> bool {
        self.method.len() > 0
    }

    /// Reset the handler back to its original state.
    pub fn reset(&mut self) {
        self.finished      = false;
        self.status_code   = 0;
        self.toggle        = true;
        self.version_major = 0;
        self.version_minor = 0;

        self.cookies.clear();
        self.field_buffer.clear();
        self.headers.clear();
        self.method.clear();
        self.status.clear();
        self.url.clear();
        self.value_buffer.clear();
    }
}

impl Http1Handler for HeadersHttp1Handler {
    fn on_header_field(&mut self, field: &[u8]) -> bool {
        if self.toggle {
            self.flush();

            self.toggle = false;
        }

        unsafe {
            self.field_buffer
                .as_mut_vec()
                .extend_from_slice(field);
        }

        true
    }

    fn on_header_value(&mut self, value: &[u8]) -> bool {
        unsafe {
            self.value_buffer
                .as_mut_vec()
                .extend_from_slice(value);
        }

        self.toggle = true;
        true
    }

    fn on_headers_finished(&mut self) -> bool {
        self.flush();

        self.finished = true;
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

    fn on_version(&mut self, major: u16, minor: u16) -> bool {
        self.version_major = major;
        self.version_minor = minor;
        true
    }
}
