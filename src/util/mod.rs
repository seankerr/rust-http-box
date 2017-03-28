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

//! Utility functions for handling encoded data, query strings, and header fields.

macro_rules! submit_error {
    ($iter:expr, $error:expr) => ({
        bs_jump!($iter.context, bs_available!($iter.context));

        (*$iter.on_error)($error($iter.context.byte));

        return None;
    });
}

macro_rules! submit_name {
    ($iter:expr) => ({
        return Some((
            unsafe {
                let mut s = String::with_capacity($iter.name.len());

                s.as_mut_vec().extend_from_slice(&$iter.name);
                s
            },
            None
        ));
    });
}
macro_rules! submit_name_value {
    ($name:expr, $value:expr) => ({
        return Some((
            unsafe {
                let mut s = String::with_capacity($name.len());

                s.as_mut_vec().extend_from_slice(&$name);
                s
            },
            unsafe {
                let mut s = String::with_capacity($value.len());

                s.as_mut_vec().extend_from_slice(&$value);
                Some(s)
            }
        ));
    });

    ($iter:expr) => ({
        return Some((
            unsafe {
                let mut s = String::with_capacity($iter.name.len());

                s.as_mut_vec().extend_from_slice(&$iter.name);
                s
            },
            unsafe {
                let mut s = String::with_capacity($iter.value.len());

                s.as_mut_vec().extend_from_slice(&$iter.value);
                Some(s)
            }
        ));
    });
}

// -------------------------------------------------------------------------------------------------

mod decode;
mod field;
mod query;

#[cfg(test)]
mod test;

pub use util::decode::{ DecodeError, decode };
pub use util::field::{ FieldError, FieldIterator };
pub use util::query::{ QueryError, QueryIterator };
