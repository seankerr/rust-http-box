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

use http1::*;
use test::http1::*;

#[test]
fn data_ok () {
    let mut p = Parser::new_multipart(DebugHandler::new());

    p.set_boundary(b"XXDebugBoundaryXX");

    assert_finished!(p,
                     b"--XXDebugBoundaryXX\r\n\r\n\
                       DATA1\r\n\
                       --XXDebugBoundaryXX\r\n\
                       Header1: Value1\r\n\
                       Header2: Value2\r\n\
                       \r\n\
                       DATA2\r\n\
                       --XXDebugBoundaryXX--");

    assert_eq!(p.handler().multipart_data,
               b"DATA1DATA2");

    assert_eq!(p.handler().header_name,
               b"header1header2");

    assert_eq!(p.handler().header_value,
               b"Value1Value2");
}

