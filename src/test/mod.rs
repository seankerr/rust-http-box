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

use byte::is_token;
use std::fmt::Debug;

mod http1;
mod util;

pub fn loop_digits<F>(skip: &[u8], function: F) where F : Fn(u8) {
    'outer:
    for n1 in 0..255 {
        for n2 in skip {
            if n1 == *n2 {
                continue 'outer;
            }
        }

        if is_digit!(n1) {
            function(n1 as u8);
        }
    }
}

pub fn loop_hex<F>(skip: &[u8], function: F) where F : Fn(u8) {
    'outer:
    for n1 in 0..255 {
        for n2 in skip {
            if n1 == *n2 {
                continue 'outer;
            }
        }

        if is_hex!(n1) {
            function(n1 as u8);
        }
    }
}

pub fn loop_non_digits<F>(skip: &[u8], function: F) where F : Fn(u8) {
    'outer:
    for n1 in 0..255 {
        for n2 in skip {
            if n1 == *n2 {
                continue 'outer;
            }
        }

        if !is_digit!(n1) {
            function(n1 as u8);
        }
    }
}

pub fn loop_non_hex<F>(skip: &[u8], function: F) where F : Fn(u8) {
    'outer:
    for n1 in 0..255 {
        for n2 in skip {
            if n1 == *n2 {
                continue 'outer;
            }
        }

        if !is_hex!(n1) {
            function(n1 as u8);
        }
    }
}

pub fn loop_non_quoted<F>(skip: &[u8], function: F) where F : Fn(u8) {
    'outer:
    for n1 in 0..255 {
        for n2 in skip {
            if n1 == *n2 {
                continue 'outer;
            }
        }

        if is_not_visible_7bit!(n1) && n1 != b' ' {
            function(n1 as u8);
        }
    }
}


pub fn loop_non_tokens<F>(skip: &[u8], function: F) where F : Fn(u8) {
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

pub fn loop_non_visible<F>(skip: &[u8], function: F) where F : Fn(u8) {
    'outer:
    for n1 in 0..255 {
        for n2 in skip {
            if n1 == *n2 {
                continue 'outer;
            }
        }

        if is_not_visible_7bit!(n1) {
            function(n1 as u8);
        }
    }
}

pub fn loop_quoted<F>(skip: &[u8], function: F) where F : Fn(u8) {
    'outer:
    for n1 in 0..255 {
        for n2 in skip {
            if n1 == *n2 {
                continue 'outer;
            }
        }

        if is_visible_7bit!(n1) || n1 == b' ' {
            function(n1 as u8);
        }
    }
}

pub fn loop_tokens<F>(skip: &[u8], function: F) where F : Fn(u8) {
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

pub fn loop_visible<F>(skip: &[u8], function: F) where F : Fn(u8) {
    'outer:
    for n1 in 0..255 {
        for n2 in skip {
            if n1 == *n2 {
                continue 'outer;
            }
        }

        if is_visible_7bit!(n1) {
            function(n1 as u8);
        }
    }
}

pub fn vec_eq<T: Debug + PartialEq>(vec: &[T], slice: &[T]) {
    assert_eq!(vec.len(), slice.len());

    for n in 0..vec.len() {
        assert_eq!(vec[n], slice[n]);
    }
}
