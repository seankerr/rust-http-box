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

use fsm::*;
use http1::*;

macro_rules! setup {
    () => ({
        let mut parser = Parser::new_chunked(DebugHandler::new());

        parser
    });
}

#[test]
fn multiple() {
    let mut p = setup!();

    assert_finished!(p,
                     b"0\r\nField1: Value1\r\nField2: Value2\r\n\r\n");

    assert!(p.handler().headers_finished);

    assert_eq!(p.handler().header_name,
               b"field1field2");

    assert_eq!(p.handler().header_value,
               b"Value1Value2");
}

#[test]
fn single() {
    let mut p = setup!();

    assert_finished!(p,
                     b"0\r\nField: Value\r\n\r\n");

    assert!(p.handler().headers_finished);

    assert_eq!(p.handler().header_name,
               b"field");

    assert_eq!(p.handler().header_value,
               b"Value");
}
