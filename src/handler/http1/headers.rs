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
use std::slice;

#[derive(Default)]
/// `HeadersHandler` is a suitable handler for the following parser functions:
///
/// - [`Parser::parse_headers()`](../http1/struct.Parser.html#method.parse_headers)
///
/// # Request Examples
///
/// ```
/// use http_box::HeadersHandler;
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
/// let mut cookie = h.cookies().get("cookie1").unwrap();
///
/// assert_eq!("value1", cookie.value().unwrap());
///
/// cookie = h.cookies().get("cookie2").unwrap();
///
/// assert_eq!("value2", cookie.value().unwrap());
/// ```
///
/// # Response Examples
///
/// ```
/// use http_box::HeadersHandler;
/// use http_box::http1::Parser;
///
/// let mut h = HeadersHandler::new();
/// let mut p = Parser::new();
///
/// p.parse_headers(&mut h,
///                 b"HTTP/1.1 200 OK\r\n\
///                   Header1: value1\r\n\
///                   Header2: value2\r\n\
///                   Set-Cookie: cookie1=value1; domain=.domain1; path=/path1\r\n\
///                   Set-Cookie: cookie2=value2; domain=.domain2; path=/path2\r\n\
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
/// let mut cookie = h.cookie("cookie1").unwrap();
///
/// assert_eq!("value1", cookie.value().unwrap());
/// assert_eq!(".domain1", cookie.domain().unwrap());
/// assert_eq!("/path1", cookie.path().unwrap());
///
/// cookie = h.cookie("cookie2").unwrap();
///
/// assert_eq!("value2", cookie.value().unwrap());
/// assert_eq!(".domain2", cookie.domain().unwrap());
/// assert_eq!("/path2", cookie.path().unwrap());
/// ```
pub struct HeadersHandler {
    /// Cookies.
    cookies: HashSet<Cookie>,

    /// Header field buffer.
    field_buffer: String,

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

impl HeadersHandler {
    /// Create a new `HeadersHandler`.
    pub fn new() -> HeadersHandler {
        HeadersHandler {
            cookies:       HashSet::new(),
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

    /// Retrieve `cookie` from the collection of cookies.
    pub fn cookie(&self, cookie: &str) -> Option<&Cookie> {
        self.cookies.get(cookie)
    }

    /// Retrieve the collection of cookies.
    pub fn cookies(&self) -> &HashSet<Cookie> {
        &self.cookies
    }

    /// Flush the most recent header field/value.
    fn flush(&mut self) {
        if self.is_request() {
            if self.field_buffer == "cookie" {
                unsafe {
                    let slice = slice::from_raw_parts(self.value_buffer.as_ptr(),
                                                      self.value_buffer.as_mut_vec().len());

                    util::parse_field(slice,
                        |s| {
                            match s {
                                FieldSegment::Name(key) => {
                                    self.cookies.insert(Cookie::new_from_slice(key));
                                },
                                FieldSegment::NameValue(key,value) => {
                                    let mut cookie = Cookie::new_from_slice(key);

                                    cookie.set_value_from_slice(value);

                                    self.cookies.insert(cookie);
                                }
                            }
                        }
                    );
                }
            } else {
                self.headers.insert(self.field_buffer.clone(), self.value_buffer.clone());
            }
        } else if self.field_buffer == "set-cookie" {
            unsafe {
                let mut cookie = Cookie::new("");

                let slice = slice::from_raw_parts(self.value_buffer.as_ptr(),
                                                  self.value_buffer.as_mut_vec().len());

                util::parse_field(slice,
                    |s| {
                        match s {
                            FieldSegment::Name(key) => {
                                if key == b"httponly" {
                                    cookie.set_http_only(true);
                                } else if key == b"secure" {
                                    cookie.set_secure(true);
                                }
                            },
                            FieldSegment::NameValue(key,value) => {
                                if key == b"domain" {
                                    cookie.set_domain_from_slice(value);
                                } else if key == b"expires" {
                                    cookie.set_expires_from_slice(value);
                                } else if key == b"max-age" {
                                    cookie.set_max_age_from_slice(value);
                                } else if key == b"path" {
                                    cookie.set_path_from_slice(value);
                                } else {
                                    cookie.set_name_from_slice(key)
                                          .set_value_from_slice(value);
                                }
                            }
                        }
                    }
                );

                if cookie.name() != "" {
                    self.cookies.insert(cookie);
                }
            }
        } else {
            self.headers.insert(self.field_buffer.clone(), self.value_buffer.clone());
        }

        self.field_buffer.clear();
        self.value_buffer.clear();
    }

    /// Indicates that `cookie` exists within the collection of cookies.
    pub fn has_cookie(&self, cookie: &str) -> bool {
        self.cookies.contains(cookie)
    }

    /// Indicates that `cookie` exists within the collection of headers.
    pub fn has_header(&self, header: &str) -> bool {
        self.headers.contains_key(header)
    }

    /// Retrieve `header` from the collection of headers.
    pub fn header(&self, header: &str) -> Option<&str> {
        if let Some(ref header) = self.headers.get(header) {
            Some(&header[..])
        } else {
            None
        }
    }

    /// Retrieve the collection of headers.
    pub fn headers(&self) -> &HashMap<String,String> {
        &self.headers
    }

    /// Indicates that header parsing has finished.
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Indicates that the parsed data is a request.
    pub fn is_request(&self) -> bool {
        !self.method.is_empty()
    }

    /// Retrieve the request method.
    pub fn method(&self) -> &String {
        &self.method
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

    /// Retrieve the response status.
    pub fn status(&self) -> &String {
        &self.status
    }

    /// Retrieve the response status code.
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    /// Retrieve the request URL.
    pub fn url(&self) -> &String {
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
