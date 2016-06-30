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

//! Support for accessing field values in an easier fashion.

use std::collections::HashMap;

/// `FieldMap` is a wrapper around `HashMap<String, FieldValue>` that provides utility
/// functions for accessing fields.
#[derive(Default)]
pub struct FieldMap(HashMap<String, FieldValue>);

impl FieldMap {
    /// Create a new `FieldMap`.
    pub fn new() -> FieldMap {
        FieldMap(HashMap::new())
    }

    /// Create a new `FieldMap` with an initial capacity.
    pub fn new_capacity(capacity: usize) -> FieldMap {
        FieldMap(HashMap::with_capacity(capacity))
    }

    /// Retrieve the internal immutable collection.
    pub fn as_map(&self) -> &HashMap<String, FieldValue> {
        &self.0
    }

    /// Retrieve the internal mutable collection.
    pub fn as_mut_map(&mut self) -> &mut HashMap<String, FieldValue> {
        &mut self.0
    }

    /// Clear all values from the collection.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Retrieve `field` from the collection.
    ///
    /// # Panic
    ///
    /// This function will panic if `field` does not exist within the collection.
    pub fn field(&self, field: &str) -> &FieldValue {
        self.0.get(field).unwrap()
    }

    /// Indicates that `field` exists within the collection.
    pub fn has_field(&self, field: &str) -> bool {
        self.0.contains_key(field)
    }

    /// Append `field` with `value` onto the collection.
    ///
    /// If `field` does not yet exist, add it.
    pub fn push(&mut self, field: &str, value: &str) {
        let mut entry = self.0.entry(field.to_string()).or_insert(FieldValue::new());

        (*entry).push(value.to_string());
    }

    /// Append `field` with `value` onto the collection.
    pub fn push_from_slice(&mut self, field: &[u8], value: &[u8]) {
        let mut n = String::with_capacity(field.len());
        let mut v = String::with_capacity(value.len());

        unsafe {
            n.as_mut_vec().extend_from_slice(field);
            v.as_mut_vec().extend_from_slice(value);
        }

        let mut entry = self.0.entry(n).or_insert(FieldValue::new());

        (*entry).push(v);
    }

    /// Remove `field` from the collection.
    pub fn remove(&mut self, field: &str) -> Option<FieldValue> {
        self.0.remove(field)
    }
}

// -------------------------------------------------------------------------------------------------

/// `FieldValue` is a wrapper around `Vec<String>` that provides utility functions for
/// accessing values.
pub struct FieldValue(Vec<String>);

impl FieldValue {
    /// Create a new `FieldValue`.
    pub fn new() -> FieldValue {
        FieldValue(Vec::new())
    }

    /// Retrieve all values from the collection.
    ///
    /// This is akin to [`as_vec()`](#method.as_vec), but it's returned as a slice.
    pub fn all(&self) -> &[String] {
        &self.0[..]
    }

    /// Retrieve the internal mutable vector.
    pub fn as_mut_vec(&mut self) -> &mut Vec<String> {
        &mut self.0
    }

    /// Retrieve the internal immutable vector.
    pub fn as_vec(&self) -> &Vec<String> {
        &self.0
    }

    /// Retrieve the first value from the collection.
    ///
    /// # Panic
    ///
    /// This function will panic if the collection is empty.
    pub fn first(&self) -> &String {
        &self.0[0]
    }

    /// Retrieve `index` from the collection.
    ///
    /// # Panic
    ///
    /// This function will panic if `index` overflows the collection length.
    pub fn get(&self, index: usize) -> &String {
        &self.0[index]
    }

    /// Indicates that the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Retrieve the number of values within the collection.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Append `value` onto the collection.
    pub fn push(&mut self, value: String) {
        self.0.push(value);
    }

    /// Remove `index` from the collection.
    ///
    /// # Panic
    ///
    /// This function will panic if `index` overflows the collection length.
    pub fn remove(&mut self, index: usize) -> String {
        self.0.remove(index)
    }
}
