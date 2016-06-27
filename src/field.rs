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
    pub fn new() -> FieldValue {
        FieldValue{ value: FieldStorage::Empty }
    }

    /// If this value is using multiple value storage, retrieve a mutable vector of values.
    ///
    /// *Note:* If this vector is cleared, the storage mechanism used internally will still reflect
    ///         multiple value storage.
    pub fn as_mut_vec(&mut self) -> Option<&mut Vec<String>> {
        match self.value {
            FieldStorage::Multiple(ref mut vec) => Some(vec),
            _ => None
        }
    }

    /// If this value is using multiple value storage, retrieve an immutable vector of values.
    pub fn as_vec(&self) -> Option<&Vec<String>> {
        match self.value {
            FieldStorage::Multiple(ref vec) => Some(vec),
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

    /// Retrieve the length of this value.
    ///
    /// If single value storage is being used, this will return `1`, not the length of the value.
    pub fn len(&self) -> usize {
        match self.value {
            FieldStorage::Single(_) => 1,
            FieldStorage::Multiple(ref vec) => vec.len(),
            _ => 0
        }
    }

    /// Push an additional value.
    ///
    /// This will silently update the internal storage mechanism to hold one or multiple
    /// values.
    pub fn push(&mut self, value: &str) {
        if self.is_empty() {
            self.value = FieldStorage::Single(value.to_string());
        } else if self.is_single() {
        } else {
            self.as_mut_vec().unwrap().push(value.to_string());
        }
    }

    /// Reset the field value back to its initial stage with empty storage.
    pub fn reset(&mut self) {
        self.value = FieldStorage::Empty
    }
}
