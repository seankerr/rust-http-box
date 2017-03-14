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
// | Author: Sean Kerr <sean@metatomic.io>                                                         |
// +-----------------------------------------------------------------------------------------------+

use http1::*;
use test::http1::*;

#[test]
fn callback_exit() {
    struct CallbackHandler;

    impl HttpHandler for CallbackHandler {
        fn on_chunk_extension_finished(&mut self) -> bool {
            false
        }
    }

    let mut h = CallbackHandler;
    let mut p = Parser::new();

    p.init_chunked();

    assert_eos!(
        p,
        h,
        b"F;extension=value",
        ChunkExtensionValue
    );

    assert_callback!(
        p,
        h,
        b";",
        StripChunkExtensionName
    );
}
