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

//! Field value support.

use std::{ fmt,
           mem };

/// Field value representation. This can be used to store values parsed via query string, URL
/// encoded values, and multipart values.
///
/// This can store zero or more values.
///
/// # Example
///
/// ```
/// use http_box::FieldValue;
///
/// let mut field = FieldValue::new();
///
/// assert!(field.is_empty());
///
/// field.push("Value1");
///
/// assert_eq!(false, field.is_empty());
/// assert_eq!(false, field.has_multiple());
/// assert_eq!("Value1", field.value().unwrap());
/// assert_eq!("Value1", field.get(0).unwrap());
///
/// field.push("Value2");
///
/// assert_eq!(false, field.is_empty());
/// assert_eq!(true, field.has_multiple());
/// assert_eq!("Value1", field.value().unwrap());
/// assert_eq!("Value1", field.get(0).unwrap());
/// assert_eq!("Value2", field.get(1).unwrap());
/// assert_eq!(None, field.get(2));
/// ```
pub struct FieldValue {
    value: FieldValueStorage
}

impl FieldValue {
    /// Create a new `FieldValue`.
    pub fn new() -> FieldValue {
        FieldValue{ value: FieldValueStorage::Empty }
    }

    /// Check to see if the storage needs downgraded to `FieldValueStorage::Empty` or
    /// `FieldValueStorage::Single`.
    fn check_storage(&mut self) {
        if self.is_empty() || !self.has_multiple() {
            return;
        }

        if self.len() == 0 {
            self.value = FieldValueStorage::Empty;
        } else if self.len() == 1 {
            self.value = FieldValueStorage::Single(self.value().unwrap().to_string());
        }
    }

    /// Clear all the values.
    pub fn clear(&mut self) {
        self.value = FieldValueStorage::Empty
    }

    /// Retrieve an iterator over the values.
    pub fn iter(&self) -> FieldValueIterator {
        FieldValueIterator{ index: 0,
                            value: &self.value }
    }

    /// Retrieve the value at `index`.
    pub fn get(&self, index: usize) -> Option<&str> {
        match self.value {
            FieldValueStorage::Single(ref string) if index == 0 => {
                Some(string)
            },
            FieldValueStorage::Multiple(ref vec) if index < vec.len() => {
                Some(&vec[index])
            },
            _ => None
        }
    }

    /// Indicates that multiple values are being stored.
    pub fn has_multiple(&self) -> bool {
        match self.value {
            FieldValueStorage::Multiple(_) => true,
            _ => false
        }
    }

    /// Indicates that this value is empty.
    pub fn is_empty(&self) -> bool {
        match self.value {
            FieldValueStorage::Empty => true,
            _ => false
        }
    }

    /// Retrieve the number of values.
    pub fn len(&self) -> usize {
        match self.value {
            FieldValueStorage::Single(_) => 1,
            FieldValueStorage::Multiple(ref vec) => vec.len(),
            _ => 0
        }
    }

    /// Append a value onto the end of the collection.
    pub fn push(&mut self, value: &str) {
        if self.is_empty() {
            self.value = FieldValueStorage::Single(value.to_string());
        } else if self.has_multiple() {
            if let FieldValueStorage::Multiple(ref mut vec) = self.value {
                vec.push(value.to_string());
            }
        } else {
            let mut old_string = String::new();

            if let FieldValueStorage::Single(ref mut string) = self.value {
                old_string = mem::replace(string, old_string);
            }

            self.value = FieldValueStorage::Multiple(vec![old_string, value.to_string()]);
        }
    }

    /// Remove the value at `index`.
    ///
    /// # Panic
    ///
    /// If `index` overflows the stored length of values.
    pub fn remove(&mut self, index: usize) {
        if self.is_empty() {
            panic!();
        }

        if self.has_multiple() {
            match self.value {
                FieldValueStorage::Multiple(ref mut vec) if index < vec.len() => {
                    vec.remove(index);
                },
                _ => {
                    panic!();
                }
            }

            self.check_storage();
        } else if index == 0 {
            self.value = FieldValueStorage::Empty;
        } else {
            panic!();
        }
    }

    /// Retain only elements allowed by `predicate`.
    pub fn retain<F>(&mut self, mut predicate: F) where F : FnMut(&String) -> bool {
        if !self.is_empty() {
            if self.has_multiple() {
                if let FieldValueStorage::Multiple(ref mut vec) = self.value {
                    vec.retain(predicate);
                }
            } else if !(if let FieldValueStorage::Single(ref string) = self.value {
                            predicate(string)
                        } else {
                            true
                        }) {
                self.value = FieldValueStorage::Empty;
            }
        }
    }

    /// Retrieve the first value.
    pub fn value(&self) -> Option<&str> {
        match self.value {
            FieldValueStorage::Single(ref string) => {
                Some(string)
            },
            FieldValueStorage::Multiple(ref vec) => {
                Some(&vec[0])
            },
            _ => None
        }
    }
}

impl fmt::Debug for FieldValue {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.value {
            FieldValueStorage::Single(ref string) => {
                write!(formatter, "FieldValueStorage::Single(\"{}\")", string)
            },
            FieldValueStorage::Multiple(ref vec) => {
                write!(formatter, "FieldValueStorage::Multiple({} values)", vec.len())
            },
            _ => write!(formatter, "FieldValueStorage::Empty")
        }
    }
}

impl fmt::Display for FieldValue {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.value {
            FieldValueStorage::Single(ref string) => {
                write!(formatter, "\"{}\"", string)
            },
            FieldValueStorage::Multiple(ref vec) => {
                write!(formatter, "{} values", vec.len())
            },
            _ => write!(formatter, "")
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Field value iterator.
pub struct FieldValueIterator<'a> {
    /// Current index.
    index: usize,

    /// Field value.
    value: &'a FieldValueStorage
}

impl<'a> Iterator for FieldValueIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;

        match self.value {
            &FieldValueStorage::Single(ref string) if self.index == 1 => {
                Some(string)
            },
            &FieldValueStorage::Multiple(ref vec) if self.index <= vec.len() => {
                Some(&vec[self.index - 1])
            },
            _ => None
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Field value storage options.
enum FieldValueStorage {
    /// Empty value.
    Empty,

    /// Multiple values.
    Multiple(Vec<String>),

    /// Single value.
    Single(String)
}
