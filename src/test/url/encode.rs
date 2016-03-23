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

use url::*;

#[test]
fn encode_without_hex() {
    assert_eq!(b"justsomedata", &encode(b"justsomedata", &mut vec![])[..]);
}

#[test]
fn encode_with_hex() {
    assert_eq!(b"just%20some%20data", &encode(b"just some data", &mut vec![])[..]);
}

#[test]
fn encode_starting_hex() {
    assert_eq!(b"%20just%20some%20data", &encode(b" just some data", &mut vec![])[..]);
}

#[test]
fn encode_ending_hex() {
    assert_eq!(b"just%20some%20data%20", &encode(b"just some data ", &mut vec![])[..]);
}

#[test]
fn encode_sequence() {
    assert_eq!(b"%20%20just%20some%20data%20%20",
               &encode(b"  just some data  ", &mut vec![])[..]);

    assert_eq!(b"just%20%20%20some%20%20%20data",
               &encode(b"just   some   data", &mut vec![])[..]);
}

#[test]
fn encode_empty() {
    assert_eq!(b"", &encode(b"", &mut vec![])[..]);
}