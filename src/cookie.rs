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

//! Cookie support.

use std::{ borrow,
           hash };
use std::fmt;

use util;
use util::{ FieldError,
            FieldSegment };

/// Cookie implementation.
#[derive(Clone,Eq,PartialEq)]
pub struct Cookie {
    /// Domain.
    domain: Option<String>,

    /// Expiration date and time.
    expires: Option<String>,

    /// Indicates the cookie is for HTTP only.
    http_only: bool,

    /// Maximum age.
    max_age: Option<String>,

    /// Name.
    name: String,

    /// Path.
    path: Option<String>,

    /// Indicates that the cookie is secure.
    secure: bool,

    /// Value.
    value: String
}

impl Cookie {
    /// Create a new `Cookie`.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_box::Cookie;
    ///
    /// let mut cookie = Cookie::new("SessionId", ":N4('<TYqK%un_yd");
    ///
    /// cookie.set_domain("rust-lang.org");
    /// cookie.set_expires("1998-10-19 20:38");
    /// cookie.set_http_only(true);
    /// cookie.set_max_age("1998-10-19 19:38");
    /// cookie.set_path("/");
    /// cookie.set_secure(true);
    ///
    /// assert_eq!("SessionId", cookie.name());
    /// assert_eq!(":N4('<TYqK%un_yd", cookie.value());
    /// assert_eq!("1998-10-19 20:38", cookie.expires().unwrap());
    /// assert_eq!("rust-lang.org", cookie.domain().unwrap());
    /// assert!(cookie.is_http_only());
    /// assert_eq!("1998-10-19 19:38", cookie.max_age().unwrap());
    /// assert_eq!("/", cookie.path().unwrap());
    /// assert!(cookie.is_secure());
    /// ```

    pub fn new<T: Into<String>>(name: T, value: T) -> Self {
        Cookie {
            domain:    None,
            expires:   None,
            http_only: false,
            max_age:   None,
            name:      name.into(),
            path:      None,
            secure:    false,
            value:     value.into()
        }
    }

    /// Create a new `Cookie` from a slice of bytes.
    ///
    /// The cookie name and value are the only required fields in order for parsing to succeed. If
    /// the cookie name or value are not present, the error will be `None`.
    ///
    /// # Unsafe
    ///
    /// This function is unsafe because it does not verify `header` is valid UTF-8.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_box::Cookie;
    ///
    /// let cookie = unsafe {
    ///     Cookie::from_bytes("Cookie=value; domain=rust-lang.org; path=/").unwrap()
    /// };
    ///
    /// assert_eq!("Cookie", cookie.name());
    /// assert_eq!("value", cookie.value());
    /// assert_eq!("rust-lang.org", cookie.domain().unwrap());
    /// assert_eq!("/", cookie.path().unwrap());
    /// ```
    pub unsafe fn from_bytes<'a, T: AsRef<[u8]>>(header: T) -> Result<Self, Option<FieldError>> {
        let bytes         = header.as_ref();
        let mut domain    = None;
        let mut expires   = None;
        let mut http_only = false;
        let mut max_age   = None;
        let mut name      = None;
        let mut path      = None;
        let mut secure    = false;
        let mut value     = None;

        // parse the name and value separately from the rest of the cookie, because we are
        // normalizing the other field names, but the cookie name cannot be
        let index = try!(util::parse_field(bytes, b';', false,
            (
                |b: u8| {
                    // cookie value cannot contain a backslash or comma per the RFC
                    !(b == b'\\' || b == b',')
                },

                |s: FieldSegment| {
                    match s {
                        FieldSegment::NameValue(n, v) => {
                            name = {
                                let mut s = String::with_capacity(n.len());

                                s.as_mut_vec().extend_from_slice(n);

                                Some(s)
                            };

                            value = {
                                let mut s = String::with_capacity(v.len());

                                s.as_mut_vec().extend_from_slice(v);

                                Some(s)
                            };
                        },
                        _ => {
                            // missing value
                        }
                    }

                    // exit parser
                    false
                }
            )
        ));

        if let None = value {
            // missing name and value
            return Err(None);
        }

        // parse the rest of the cookie details
        try!(util::parse_field(&bytes[index..], b';', true,
            |s: FieldSegment| {
                match s {
                    FieldSegment::Name(name) => {
                        if name == b"httponly" {
                            http_only = true;
                        } else if name == b"secure" {
                            secure = true;
                        }
                    },
                    FieldSegment::NameValue(name, value) => {
                        if name == b"domain" {
                            domain = {
                                let mut s = String::with_capacity(value.len());

                                s.as_mut_vec().extend_from_slice(value);

                                Some(s)
                            };
                        } else if name == b"expires" {
                            expires = {
                                let mut s = String::with_capacity(value.len());

                                s.as_mut_vec().extend_from_slice(value);

                                Some(s)
                            };
                        } else if name == b"max-age" {
                            max_age = {
                                let mut s = String::with_capacity(value.len());

                                s.as_mut_vec().extend_from_slice(value);

                                Some(s)
                            };
                        } else if name == b"path" {
                            path = {
                                let mut s = String::with_capacity(value.len());

                                s.as_mut_vec().extend_from_slice(value);

                                Some(s)
                            };
                        }
                    }
                }

                true
            }
        ));

        Ok(Cookie {
            domain:    domain,
            expires:   expires,
            http_only: http_only,
            max_age:   max_age,
            name:      name.unwrap(),
            path:      path,
            secure:    secure,
            value:     value.unwrap()
        })
    }

    /// Create a new `Cookie` from a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_box::Cookie;
    ///
    /// let cookie = Cookie::from_str("Cookie=value; domain=rust-lang.org; path=/").unwrap();
    ///
    /// assert_eq!("Cookie", cookie.name());
    /// assert_eq!("value", cookie.value());
    /// assert_eq!("rust-lang.org", cookie.domain().unwrap());
    /// assert_eq!("/", cookie.path().unwrap());
    /// ```
    pub fn from_str<T: AsRef<str>>(string: T) -> Result<Self, Option<FieldError>> {
        unsafe {
            Cookie::from_bytes(string.as_ref())
        }
    }

    /// Retrieve the domain.
    pub fn domain(&self) -> Option<&str> {
        if let Some(ref x) = self.domain {
            Some(x)
        } else {
            None
        }
    }

    /// Retrieve the expiration date and time.
    pub fn expires(&self) -> Option<&str> {
        if let Some(ref x) = self.expires {
            Some(x)
        } else {
            None
        }
    }

    /// Indicates that the cookie is for HTTP only.
    pub fn is_http_only(&self) -> bool {
        self.http_only
    }

    /// Indicates that the cookie is secure.
    pub fn is_secure(&self) -> bool {
        self.secure
    }

    /// Retrieve the maximum age.
    pub fn max_age(&self) -> Option<&str> {
        if let Some(ref x) = self.max_age {
            Some(x)
        } else {
            None
        }
    }

    /// Retrieve the name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Retrieve the path.
    pub fn path(&self) -> Option<&str> {
        if let Some(ref x) = self.path {
            Some(x)
        } else {
            None
        }
    }

    /// Set the domain.
    pub fn set_domain<T: Into<String>>(&mut self, domain: T) -> &mut Self {
        self.domain = Some(domain.into());
        self
    }

    /// Set the expiration date and time.
    pub fn set_expires<T: Into<String>>(&mut self, expires: T) -> &mut Self {
        self.expires = Some(expires.into());
        self
    }

    /// Set the HTTP only status.
    pub fn set_http_only(&mut self, http_only: bool) -> &mut Self {
        self.http_only = http_only;
        self
    }

    /// Set the maximum age.
    pub fn set_max_age<T: Into<String>>(&mut self, max_age: T) -> &mut Self {
        self.max_age = Some(max_age.into());
        self
    }

    /// Set the name.
    pub fn set_name<T: Into<String>>(&mut self, name: T) -> &mut Self {
        self.name = name.into();
        self
    }

    /// Set the path.
    pub fn set_path<T: Into<String>>(&mut self, path: T) -> &mut Self {
        self.path = Some(path.into());
        self
    }

    /// Set the secure status.
    pub fn set_secure(&mut self, secure: bool) -> &mut Self {
        self.secure = secure;
        self
    }

    /// Set the value.
    pub fn set_value<T: Into<String>>(&mut self, value: T) -> &mut Self {
        self.value = value.into();
        self
    }

    /// Retrieve the value.
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl borrow::Borrow<str> for Cookie {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl fmt::Debug for Cookie {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter,
               "Cookie(name=\"{}\", value=\"{}\", domain=\"{}\", path=\"{}\", \
                       expires=\"{}\", max-age=\"{}\", http-only={}, secure={})",
               self.name,
               self.value,
               if let Some(ref s) = self.domain { &s[..] } else { "" },
               if let Some(ref s) = self.path { &s[..] } else { "" },
               if let Some(ref s) = self.expires { &s[..] } else { "" },
               if let Some(ref s) = self.max_age { &s[..] } else { "" },
               self.http_only,
               self.secure)
    }
}

impl fmt::Display for Cookie {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.value)
    }
}

impl hash::Hash for Cookie {
    #[inline]
    fn hash<H>(&self, state: &mut H) where H : hash::Hasher {
        self.name.hash(state)
    }
}

impl PartialEq<str> for Cookie {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.name == other
    }
}
