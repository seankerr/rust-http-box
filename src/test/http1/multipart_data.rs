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

use handler::*;
use http1::*;
use test::http1::*;

#[test]
fn data_ok () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    multipart_assert_finished(&mut p, &mut h,
                              b"--XXDebugBoundaryXX\r\n\r\n\
                                DATA1\r\n\
                                --XXDebugBoundaryXX\r\n\
                                Header1: Value1\r\n\
                                Header2: Value2\r\n\
                                \r\n\
                                DATA2\r\n\
                                --XXDebugBoundaryXX--\r\n",
                         ParserState::Finished, 117);

    assert_eq!(h.multipart_data, b"DATA1DATA2");
    assert_eq!(h.header_field, b"header1header2");
    assert_eq!(h.header_value, b"Value1Value2");
}

