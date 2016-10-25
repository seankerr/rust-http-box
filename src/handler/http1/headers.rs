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

use cookie::Cookie;
use http1::Http1Handler;
use util::FieldSegment;
use util;

use std::collections::{ HashMap,
                        HashSet };
use std::str;

#[derive(Default)]
/// `HeadersHandler` is a suitable handler for the following parser functions:
///
/// - [`Parser::parse_headers()`](../http1/struct.Parser.html#method.parse_headers)
///
/// # Request Examples
///
/// ```
/// use http_box::handler::HeadersHandler;
/// use http_box::http1::Parser;
///
/// let mut h = HeadersHandler::new();
/// let mut p = Parser::new();
///
/// p.parse_headers(&mut h,
///                 b"GET / HTTP/1.1\r\n\
///                   Header1: value1\r\n\
///                   Header2: value2\r\n\
///                   Cookie: Cookie1=value1; Cookie2=value2\r\n\
///                   \r\n\r\n", 0);
///
/// // header fields are normalized to lower-case
/// assert_eq!("value1", h.headers().get("header1").unwrap());
/// assert_eq!("value2", h.headers().get("header2").unwrap());
///
/// // request details
/// assert_eq!("GET", h.method());
/// assert_eq!("/", h.url());
/// assert_eq!(1, h.version_major());
/// assert_eq!(1, h.version_minor());
///
/// // cookie names are normalized to lower-case
/// let mut cookie = h.cookies().get("Cookie1").unwrap();
///
/// assert_eq!("value1", cookie.value());
///
/// cookie = h.cookies().get("Cookie2").unwrap();
///
/// assert_eq!("value2", cookie.value());
/// ```
///
/// # Response Examples
///
/// ```
/// use http_box::handler::HeadersHandler;
/// use http_box::http1::Parser;
///
/// let mut h = HeadersHandler::new();
/// let mut p = Parser::new();
///
/// p.parse_headers(&mut h,
///                 b"HTTP/1.1 200 OK\r\n\
///                   Header1: value1\r\n\
///                   Header2: value2\r\n\
///                   Set-Cookie: Cookie1=value1; domain=.domain1; path=/path1\r\n\
///                   Set-Cookie: Cookie2=value2; domain=.domain2; path=/path2\r\n\
///                   \r\n\r\n", 0);
///
/// // header fields are normalized to lower-case
/// assert_eq!("value1", h.headers().get("header1").unwrap());
/// assert_eq!("value2", h.headers().get("header2").unwrap());
///
/// // response details
/// assert_eq!(1, h.version_major());
/// assert_eq!(1, h.version_minor());
/// assert_eq!(200, h.status_code());
/// assert_eq!("OK", h.status());
///
/// // cookie names are normalized to lower-case
/// let mut cookie = h.cookie("Cookie1").unwrap();
///
/// assert_eq!("value1", cookie.value());
/// assert_eq!(".domain1", cookie.domain().unwrap());
/// assert_eq!("/path1", cookie.path().unwrap());
///
/// cookie = h.cookie("Cookie2").unwrap();
///
/// assert_eq!("value2", cookie.value());
/// assert_eq!(".domain2", cookie.domain().unwrap());
/// assert_eq!("/path2", cookie.path().unwrap());
/// ```
pub struct HeadersHandler {
    /// Cookies.
    cookies: HashSet<Cookie>,

    /// Header field buffer.
    field_buffer: Vec<u8>,

    /// Indicates that header parsing has finished.
    finished: bool,

    /// Headers.
    headers: HashMap<String, String>,

    /// Request method.
    method: String,

    /// Response status.
    status: String,

    /// Response status code.
    status_code: u16,

    /// Field/value toggle.
    toggle: bool,

    /// Request URL.
    url: String,

    /// Header value buffer.
    value_buffer: Vec<u8>,

    /// HTTP major version.
    version_major: u16,

    /// HTTP minor version.
    version_minor: u16
}

impl HeadersHandler {
    /// Create a new `HeadersHandler`.
    pub fn new() -> HeadersHandler {
        HeadersHandler {
            cookies:       HashSet::new(),
            field_buffer:  Vec::with_capacity(10),
            finished:      false,
            headers:       HashMap::new(),
            method:        String::with_capacity(0),
            status:        String::with_capacity(0),
            status_code:   0,
            toggle:        false,
            url:           String::with_capacity(0),
            value_buffer:  Vec::with_capacity(10),
            version_major: 0,
            version_minor: 0
        }
    }

    /// Retrieve `cookie` from the collection of cookies.
    pub fn cookie<T: AsRef<str>>(&self, cookie: T) -> Option<&Cookie> {
        self.cookies.get(cookie.as_ref())
    }

    /// Retrieve the collection of cookies.
    pub fn cookies(&self) -> &HashSet<Cookie> {
        &self.cookies
    }

    /// Flush the most recent header field/value.
    fn flush(&mut self) {
        if self.field_buffer == b"cookie" {
            let buffer = self.value_buffer.clone();

            util::parse_field(&buffer, b';', false,
                |s: FieldSegment| {
                    match s {
                        FieldSegment::NameValue(name, value) => {
                            self.cookies.insert(Cookie::new(
                                // name
                                unsafe {
                                    let mut s = String::with_capacity(name.len());

                                    s.as_mut_vec().extend_from_slice(name);
                                    s
                                },

                                // value
                                unsafe {
                                    let mut s = String::with_capacity(value.len());

                                    s.as_mut_vec().extend_from_slice(value);
                                    s
                                }
                            ));

                            true
                        },
                        _ => {
                            // missing value
                            false
                        }
                    }
                }
            );
        } else if self.field_buffer == b"set-cookie" {
            match Cookie::from_bytes(self.value_buffer.as_slice()) {
                Ok(cookie) => {
                    self.cookies.insert(cookie);
                },
                _ => {
                    // invalid cookie headder
                }
            }
        } else {
            self.headers.insert(
                // name
                unsafe {
                    let mut s = String::with_capacity(self.field_buffer.len());

                    s.as_mut_vec().extend_from_slice(&self.field_buffer);
                    s
                },

                // value
                unsafe {
                    let mut s = String::with_capacity(self.value_buffer.len());

                    s.as_mut_vec().extend_from_slice(&self.value_buffer);
                    s
                }
            );
        }

        self.field_buffer.clear();
        self.value_buffer.clear();
    }

    /// Indicates that `cookie` exists within the collection of cookies.
    pub fn has_cookie<T: AsRef<str>>(&self, cookie: T) -> bool {
        self.cookies.contains(cookie.as_ref())
    }

    /// Indicates that `header` exists within the collection of headers.
    pub fn has_header<T: AsRef<str>>(&self, header: T) -> bool {
        self.headers.contains_key(header.as_ref())
    }

    /// Retrieve `header` from the collection of headers.
    pub fn header<T: AsRef<str>>(&self, header: T) -> Option<&str> {
        if let Some(header) = self.headers.get(header.as_ref()) {
            Some(&header[..])
        } else {
            None
        }
    }

    /// Retrieve the collection of headers.
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Indicates that header parsing has finished.
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Indicates that the parsed data is a request.
    pub fn is_request(&self) -> bool {
        self.status.is_empty()
    }

    /// Retrieve the request method.
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Reset the handler to its original state.
    pub fn reset(&mut self) {
        self.finished      = false;
        self.status_code   = 0;
        self.toggle        = false;
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

    /// Retrieve the response status.
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Retrieve the response status code.
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    /// Retrieve the request URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Retrieve the HTTP major version.
    pub fn version_major(&self) -> u16 {
        self.version_major
    }

    /// Retrieve the HTTP minor version.
    pub fn version_minor(&self) -> u16 {
        self.version_minor
    }
}

impl Http1Handler for HeadersHandler {
    fn content_length(&mut self) -> Option<usize> {
        if let Some(content_length) = self.header("content-length") {
            let mut length: usize = 0;

            for byte in content_length.as_bytes().iter() {
                if is_digit!(*byte) {
                    if let Some(num) = length.checked_mul(10) {
                        if let Some(num) = num.checked_add((*byte - b'0') as usize) {
                            length = num;
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }

                } else {
                    // contains non-digit
                    return None;
                }
            }

            Some(length)
        } else {
            None
        }
    }

    fn on_header_field(&mut self, field: &[u8]) -> bool {
        if self.toggle {
            self.flush();

            self.toggle = false;
        }

        self.field_buffer.extend_from_slice(field);

        true
    }

    fn on_header_value(&mut self, value: &[u8]) -> bool {
        self.value_buffer.extend_from_slice(value);

        self.toggle = true;
        true
    }

    fn on_headers_finished(&mut self) -> bool {
        self.flush();
        self.finished = true;

        if self.is_request() {
            self.method.shrink_to_fit();
            self.url.shrink_to_fit();
        } else {
            self.status.shrink_to_fit();
        }

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
