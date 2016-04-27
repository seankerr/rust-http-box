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

use Success;
use http1::{ HttpHandler,
             Parser,
             State };
use url::ParamHandler;
use std::fmt::Debug;

mod byte;
mod http1;
mod url;

fn assert_vec_eq<T: Debug + PartialEq>(vec: Vec<T>, slice: &[T]) {
    assert_eq!(vec.len(), slice.len());

    for n in 0..vec.len() {
        assert_eq!(vec[n], slice[n]);
    }
}

fn setup<T:HttpHandler + ParamHandler>(p: &mut Parser<T>, h: &mut T, data: &[u8], state: State) {
    assert!(match p.parse(h, data) {
        Ok(Success::Eof(length)) => {
            assert_eq!(length, data.len());
            assert_eq!(p.get_state(), state);
            true
        },
        _ => false
    });
}
