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

//! Support for accessing parameter values in an easier fashion.

use std::collections::HashMap;

/// `ParameterMap` is a wrapper around `HashMap<String, ParameterValue>` that provides utility
/// functions for accessing parameters.
///
/// # Examples
///
/// ```
/// use http_box::parameter::ParameterMap;
///
/// let mut map = ParameterMap::new();
///
/// map.push("key", "value1");
/// map.push("key", "value2");
///
/// assert_eq!(1, map.len());
/// assert_eq!(2, map.parameter("key").unwrap().len());
///
/// assert_eq!("value1", map.parameter("key").unwrap().first().unwrap());
/// assert_eq!("value1", map.parameter("key").unwrap().get(0).unwrap());
/// assert_eq!("value2", map.parameter("key").unwrap().get(1).unwrap());
///
/// map.parameter_mut("key").unwrap().remove(1);
///
/// assert_eq!(1, map.parameter("key").unwrap().len());
///
/// map.remove("key");
///
/// assert_eq!(false, map.has_parameter("key"));
/// ```
#[derive(Default)]
pub struct ParameterMap(HashMap<String, ParameterValue>);

impl ParameterMap {
    /// Create a new `ParameterMap`.
    pub fn new() -> Self {
        ParameterMap(HashMap::new())
    }

    /// Create a new `ParameterMap` with an initial capacity of `capacity`.
    pub fn with_capacity(capacity: usize) -> Self {
        ParameterMap(HashMap::with_capacity(capacity))
    }

    /// Retrieve the internal immutable collection.
    pub fn as_map(&self) -> &HashMap<String, ParameterValue> {
        &self.0
    }

    /// Retrieve the internal mutable collection.
    pub fn as_mut_map(&mut self) -> &mut HashMap<String, ParameterValue> {
        &mut self.0
    }

    /// Clear the collection.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Indicates that `parameter` exists within the collection.
    pub fn has_parameter<T: AsRef<str>>(&self, parameter: T) -> bool {
        self.0.contains_key(parameter.as_ref())
    }

    /// Indicates that the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Retrieve the number of parameters within the collection.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Retrieve immutable `parameter` from the collection.
    pub fn parameter<T: AsRef<str>>(&self, parameter: T) -> Option<&ParameterValue> {
        self.0.get(parameter.as_ref())
    }

    /// Retrieve mutable `parameter` from the collection.
    pub fn parameter_mut<T: AsRef<str>>(&mut self, parameter: T) -> Option<&mut ParameterValue> {
        self.0.get_mut(parameter.as_ref())
    }

    /// Append `parameter` with `value` onto the collection.
    ///
    /// If `parameter` does not yet exist, add it.
    pub fn push<T: Into<String>>(&mut self, parameter: T, value: T) -> &mut Self {
        {
            let mut entry = self.0.entry(parameter.into()).or_insert(ParameterValue::new());

            (*entry).push(value.into());
        }

        self
    }

    /// Remove `parameter` from the collection.
    pub fn remove<T: AsRef<str>>(&mut self, parameter: T) -> Option<ParameterValue> {
        self.0.remove(parameter.as_ref())
    }
}

// -------------------------------------------------------------------------------------------------

/// `ParameterValue` is a wrapper around `Vec<String>` that provides utility functions for accessing
/// values.
#[derive(Default)]
pub struct ParameterValue(Vec<String>);

impl ParameterValue {
    /// Create a new `ParameterValue`.
    pub fn new() -> Self {
        ParameterValue(Vec::new())
    }

    /// Retrieve all values from the collection.
    ///
    /// This is akin to [`&as_vec()[..]`](#method.as_vec).
    pub fn all(&self) -> &[String] {
        &self.0[..]
    }

    /// Retrieve the internal mutable collection.
    pub fn as_mut_vec(&mut self) -> &mut Vec<String> {
        &mut self.0
    }

    /// Retrieve the internal immutable collection.
    pub fn as_vec(&self) -> &Vec<String> {
        &self.0
    }

    /// Retrieve the first value from the collection.
    pub fn first(&self) -> Option<&str> {
        if self.0.is_empty() {
            None
        } else {
            Some(&self.0[0])
        }
    }

    /// Retrieve `index` from the collection.
    pub fn get(&self, index: usize) -> Option<&str> {
        if self.0.is_empty() {
            None
        } else {
            Some(&self.0[index])
        }
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
    pub fn push<T: Into<String>>(&mut self, value: T) -> &mut Self {
        self.0.push(value.into());
        self
    }

    /// Remove `index` from the collection and return it.
    pub fn remove(&mut self, index: usize) -> Option<String> {
        if index < self.0.len() {
            Some(self.0.remove(index))
        } else {
            None
        }
    }
}
