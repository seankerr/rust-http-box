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

#[macro_export]
macro_rules! query {
    ($stream:expr, $field:expr, $value:expr, $has_field:expr, $has_value:expr,
     $has_flushed:expr, $length:expr) => ({
        let mut field       = vec![];
        let mut has_field   = false;
        let mut has_flushed = false;
        let mut has_value   = false;
        let mut value       = vec![];

        assert!(match parse_query_string($stream,
                                         |segment| {
                                             match segment {
                                                 QuerySegment::Field(x) => {
                                                     has_field = true;
                                                     field.extend_from_slice(x)
                                                 },
                                                 QuerySegment::Flush => {
                                                     has_flushed = true
                                                 },
                                                 QuerySegment::Value(x) => {
                                                     has_value = true;
                                                     value.extend_from_slice(x)
                                                 }
                                             }
                                         }) {
            Ok($length) => {
                assert_eq!(field, $field);
                assert_eq!(value, $value);
                assert_eq!(has_field, $has_field);
                assert_eq!(has_value, $has_value);
                assert_eq!(has_flushed, $has_flushed);
                true
            },
            _ => false
        });
    });
}

mod decode;
mod parse_query_string_field;
mod parse_query_string_value;
mod parse_url;
