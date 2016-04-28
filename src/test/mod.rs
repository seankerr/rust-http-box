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
use byte::is_token;
use http1::{ HttpHandler,
             Parser,
             State };
use url::ParamHandler;
use std::fmt::Debug;

mod byte;
mod http1;
mod url;

fn loop_control<F>(skip: &[u8], function: F) where F : Fn(u8) {
    'outer:
    for n1 in 0..255 {
        for n2 in skip {
            if n1 == *n2 {
                continue 'outer;
            }
        }

        if !is_ascii!(n1) || is_control!(n1) {
            function(n1 as u8);
        }
    }
}

fn loop_non_control<F>(skip: &[u8], function: F) where F : Fn(u8) {
    'outer:
    for n1 in 0..255 {
        for n2 in skip {
            if n1 == *n2 {
                continue 'outer;
            }
        }

        if is_ascii!(n1) && !is_control!(n1) {
            function(n1 as u8);
        }
    }
}

fn loop_non_tokens<F>(skip: &[u8], function: F) where F : Fn(u8) {
    'outer:
    for n1 in 0..255 {
        for n2 in skip {
            if n1 == *n2 {
                continue 'outer;
            }
        }

        if !is_token(n1 as u8) {
            function(n1 as u8);
        }
    }
}

fn loop_tokens<F>(skip: &[u8], function: F) where F : Fn(u8) {
    'outer:
    for n1 in 0..255 {
        for n2 in skip {
            if n1 == *n2 {
                continue 'outer;
            }
        }

        if is_token(n1 as u8) {
            function(n1 as u8);
        }
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

fn vec_eq<T: Debug + PartialEq>(vec: Vec<T>, slice: &[T]) {
    assert_eq!(vec.len(), slice.len());

    for n in 0..vec.len() {
        assert_eq!(vec[n], slice[n]);
    }
}
