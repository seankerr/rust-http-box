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
fn headers() {
    let (mut p, mut h) = http1_setup!();

    p.init_multipart();
    p.set_boundary(b"XTestBoundaryX");

    assert_eos(
        &mut p,
        &mut h,
        b"--XTestBoundaryX\r\n\
          Header1: Value1\r\n\
          Header2: Value2\r\n",
        ParserState::HeaderCr2,
        b"--XTestBoundaryX\r\n\
          Header1: Value1\r\n\
          Header2: Value2\r\n".len()
    );

    assert_eq!(
        &h.header_name,
        b"header1header2"
    );

    assert_eq!(
        &h.header_value,
        b"Value1Value2"
    );
}
