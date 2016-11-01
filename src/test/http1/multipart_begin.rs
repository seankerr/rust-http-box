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
fn callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn multipart_boundary(&mut self) -> Option<&[u8]> {
            Some(b"XXDebugBoundaryXX")
        }

        fn on_multipart_begin(&mut self) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new();

    multipart_assert_callback(&mut p, &mut h,
                              b"--XXDebugBoundaryXX\r",
                              ParserState::PreHeaders1, 20);

}