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

use http1::*;
use http1::test::*;

#[test]
fn finished() {
    let (mut p, mut h) = http1_setup!();

    p.init_url_encoded();
    p.set_length(b"Name+1%21=Value%201%21".len());

    assert_finished(
        &mut p,
        &mut h,
        b"Name+1%21=Value%201%21",
        b"Name+1%21=Value%201%21".len()
    );

    assert_eq!(
        &h.url_encoded_name,
        b"Name 1!"
    );

    assert_eq!(
        &h.url_encoded_value,
        b"Value 1!"
    );
}
