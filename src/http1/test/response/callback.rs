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
fn on_status() {
    struct H;
    impl HttpHandler for H {
        fn on_status(&mut self, _: &[u8]) -> bool {
            false
        }
    }

    let mut h = H;
    let mut p = Parser::new();

    assert_callback(
        &mut p,
        &mut h,
        b"HTTP/1.1 200 X",
        ParserState::ResponseStatus2,
        b"HTTP/1.1 200 X".len()
    );
}

#[test]
fn on_status_code() {
    struct H;
    impl HttpHandler for H {
        fn on_status_code(&mut self, _: u16) -> bool {
            false
        }
    }

    let mut h = H;
    let mut p = Parser::new();

    assert_callback(
        &mut p,
        &mut h,
        b"HTTP/1.1 200 ",
        ParserState::ResponseStatus1,
        b"HTTP/1.1 200 ".len()
    );
}

#[test]
fn on_version() {
    struct H;
    impl HttpHandler for H {
        fn on_version(&mut self, _: u16, _: u16) -> bool {
            false
        }
    }

    let mut h = H;
    let mut p = Parser::new();

    assert_callback(
        &mut p,
        &mut h,
        b"HTTP/1.1 ",
        ParserState::ResponseStatusCode1,
        b"HTTP/1.1 ".len()
    );
}
