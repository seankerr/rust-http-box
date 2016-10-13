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
use std::fmt::{ Debug,
                Display,
                Formatter,
                Result };

/// HTTP cookie.
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
    value: Option<String>
}

impl Cookie {
    /// Create a new `Cookie`.
    pub fn new(name: &str) -> Self {
        Cookie{
            domain:    None,
            expires:   None,
            http_only: false,
            max_age:   None,
            name:      name.to_string(),
            path:      None,
            secure:    false,
            value:     None
        }
    }

    /// Create a new `Cookie`.
    pub unsafe fn new_from_slice(name: &[u8]) -> Self {
        Cookie{
            domain:    None,
            expires:   None,
            http_only: false,
            max_age:   None,
            name:      {
                let mut s = String::with_capacity(name.len());

                s.as_mut_vec().extend_from_slice(name);
                s
            },
            path:      None,
            secure:    false,
            value:     None
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
    pub fn name(&self) -> &String {
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
    pub fn set_domain(&mut self, domain: String) -> &mut Self {
        self.domain = Some(domain);
        self
    }

    /// Set the domain.
    pub unsafe fn set_domain_from_slice(&mut self, domain: &[u8]) -> &mut Self {
        self.domain = Some({
            let mut s = String::with_capacity(domain.len());

            s.as_mut_vec().extend_from_slice(domain);
            s
        });

        self
    }

    /// Set the expiration date and time.
    pub fn set_expires(&mut self, expires: String) -> &mut Self {
        self.expires = Some(expires);
        self
    }

    /// Set the expiration date and time.
    pub unsafe fn set_expires_from_slice(&mut self, expires: &[u8]) -> &mut Self {
        self.expires = Some({
            let mut s = String::with_capacity(expires.len());

            s.as_mut_vec().extend_from_slice(expires);
            s
        });

        self
    }

    /// Set the HTTP only status.
    pub fn set_http_only(&mut self, http_only: bool) -> &mut Self {
        self.http_only = http_only;
        self
    }

    /// Set the maximum age.
    pub fn set_max_age(&mut self, max_age: String) -> &mut Self {
        self.max_age = Some(max_age);
        self
    }

    /// Set the maximum age.
    pub unsafe fn set_max_age_from_slice(&mut self, max_age: &[u8]) -> &mut Self {
        self.max_age = Some({
            let mut s = String::with_capacity(max_age.len());

            s.as_mut_vec().extend_from_slice(max_age);
            s
        });

        self
    }

    /// Set the name.
    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }

    /// Set the name.
    pub unsafe fn set_name_from_slice(&mut self, name: &[u8]) -> &mut Self {
        self.name = {
            let mut s = String::with_capacity(name.len());

            s.as_mut_vec().extend_from_slice(name);
            s
        };

        self
    }

    /// Set the path.
    pub fn set_path(&mut self, path: String) -> &mut Self {
        self.path = Some(path);
        self
    }

    /// Set the path.
    pub unsafe fn set_path_from_slice(&mut self, path: &[u8]) -> &mut Self {
        self.path = Some({
            let mut s = String::with_capacity(path.len());

            s.as_mut_vec().extend_from_slice(path);
            s
        });

        self
    }

    /// Set the secure status.
    pub fn set_secure(&mut self, secure: bool) -> &mut Self {
        self.secure = secure;
        self
    }

    /// Set the value.
    pub fn set_value(&mut self, value: String) -> &mut Self {
        self.value = Some(value);
        self
    }

    /// Set the value.
    pub unsafe fn set_value_from_slice(&mut self, value: &[u8]) -> &mut Self {
        self.value = Some({
            let mut s = String::with_capacity(value.len());

            s.as_mut_vec().extend_from_slice(value);
            s
        });

        self
    }

    /// Retrieve the value.
    pub fn value(&self) -> Option<&str> {
        if let Some(ref x) = self.value {
            Some(x)
        } else {
            None
        }
    }
}

impl borrow::Borrow<str> for Cookie {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl Debug for Cookie {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        write!(formatter,
               "Cookie(name=\"{}\", value=\"{}\", domain=\"{}\", path=\"{}\", \
                       expires=\"{}\", max-age=\"{}\", http-only={}, secure={})",
               self.name,
               if let Some(ref s) = self.value { &s[..] } else { "" },
               if let Some(ref s) = self.domain { &s[..] } else { "" },
               if let Some(ref s) = self.path { &s[..] } else { "" },
               if let Some(ref s) = self.expires { &s[..] } else { "" },
               if let Some(ref s) = self.max_age { &s[..] } else { "" },
               self.http_only,
               self.secure)
    }
}

impl Display for Cookie {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        write!(formatter, "{}", if let Some(ref s) = self.value { &s[..] } else { "" })
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
