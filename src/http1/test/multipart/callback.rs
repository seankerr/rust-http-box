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
fn on_multipart_begin() {
    struct H;
    impl HttpHandler for H {
        fn on_multipart_begin(&mut self) -> bool {
            false
        }
    }

    let mut h = H;
    let mut p = Parser::new();

    p.init_multipart();
    p.set_boundary(b"XTestBoundaryX");

    assert_callback(
        &mut p,
        &mut h,
        b"--XTestBoundaryX\r",
        ParserState::InitialLf,
        b"--XTestBoundaryX\r".len()
    );
}

#[test]
fn on_multipart_data() {
    struct H;
    impl HttpHandler for H {
        fn on_multipart_data(&mut self, _: &[u8]) -> bool {
            false
        }
    }

    let mut h = H;
    let mut p = Parser::new();

    p.init_multipart();
    p.set_boundary(b"XTestBoundaryX");

    assert_callback(
        &mut p,
        &mut h,
        b"--XTestBoundaryX\r\n\
         Header: Value\r\n\
         \r\n\
         Data",
        ParserState::MultipartDataByByte,
        b"--XTestBoundaryX\r\n\
         Header: Value\r\n\
         \r\n\
         Data".len()
    );
}
