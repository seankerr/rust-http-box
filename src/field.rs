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

//! Field/value support.

/// Field storage types.
enum FieldStorage {
    /// Empty storage.
    Empty,

    /// Multiple values.
    Multiple(Vec<String>),

    /// Single value.
    Single(String)
}

/// Field value representation, such as query string field/value pairs, URL encoded field/value
/// pairs, and multipart field/value pairs.
///
/// This can store single or multiple values.
pub struct FieldValue {
    value: FieldStorage
}

impl FieldValue {
    /// Create a new `FieldValue`.
    pub fn new(value: &str) -> FieldValue {
        FieldValue{ value: FieldStorage::Single(value.to_string()) }
    }

    /// Create a new `FieldValue` from slice.
    pub fn new_from_slice(value: &[u8]) -> FieldValue {
        let mut string = String::with_capacity(value.len());

        unsafe {
            string.as_mut_vec().extend_from_slice(value);
        }

        FieldValue{ value: FieldStorage::Single(string) }
    }

    /// If this value is using multiple value storage, retrieve a mutable vector of values.
    pub fn get_mut_vec(&mut self) -> Option<&mut Vec<String>> {
        match self.value {
            FieldStorage::Multiple(ref mut vec) => Some(vec),
            _ => None
        }
    }

    /// If this value is using single value storage, retrieve the value.
    pub fn get_value(&self) -> Option<&str> {
        match self.value {
            FieldStorage::Single(ref string) => Some(string),
            _ => None
        }
    }

    /// If this value is using multiple value storage, retrieve an immutable vector of values.
    pub fn get_vec(&self) -> Option<&Vec<String>> {
        match self.value {
            FieldStorage::Multiple(ref vec) => Some(vec),
            _ => None
        }
    }

    /// Indicates that this value is empty.
    pub fn is_empty(&self) -> bool {
        match self.value {
            FieldStorage::Empty => true,
            _ => false
        }
    }

    /// Indicates that multiple storage is being used to hold values.
    pub fn is_multiple(&self) -> bool {
        match self.value {
            FieldStorage::Multiple(_) => true,
            _ => false
        }
    }

    /// Indicates that single storage is being used to hold values.
    pub fn is_single(&self) -> bool {
        match self.value {
            FieldStorage::Single(_) => true,
            _ => false
        }
    }

    /// Push an additional value.
    ///
    /// If the initial value was single value storage, it will be updated to multiple value
    /// storage.
    pub fn push(&mut self, value: &str) {
        if self.is_single() {
        } else if self.is_multiple() {
            self.get_mut_vec().unwrap().push(value.to_string());
        } else {
            self.value = FieldStorage::Single(value.to_string());
        }
    }
}
