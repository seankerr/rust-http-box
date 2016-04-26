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
    let mut vec = vec![];

    encode(b"justsomedata", &mut vec);

    assert_eq!(vec, b"justsomedata");
}

#[test]
fn encode_with_hex() {
    let mut vec = vec![];

    encode(b"just some data", &mut vec);

    assert_eq!(vec, b"just%20some%20data");
}

#[test]
fn encode_starting_hex() {
    let mut vec = vec![];

    encode(b" just some data", &mut vec);

    assert_eq!(vec, b"%20just%20some%20data");
}

#[test]
fn encode_ending_hex() {
    let mut vec = vec![];

    encode(b"just some data ", &mut vec);

    assert_eq!(vec, b"just%20some%20data%20");
}

#[test]
fn encode_sequence() {
    let mut vec = vec![];

    encode(b"  just some data  ", &mut vec);

    assert_eq!(vec, b"%20%20just%20some%20data%20%20");

    vec.clear();

    encode(b"just   some   data", &mut vec);

    assert_eq!(vec, b"just%20%20%20some%20%20%20data");
}

#[test]
fn encode_empty() {
    let mut vec = vec![];

    encode(b"", &mut vec);

    assert_eq!(vec, b"");
}
