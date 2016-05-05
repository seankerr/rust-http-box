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
             ParserError,
             State,
             StateFunction };
use url::ParamHandler;
use std::fmt::Debug;

mod byte;
mod http1;
mod url;

pub fn assert_callback<T: HttpHandler + ParamHandler>(parser: &mut Parser<T>, handler: &mut T,
                                                      stream: &[u8], state: State, length: usize) {
    assert!(match parser.parse(handler, stream) {
        Ok(Success::Callback(byte_count)) => {
            assert_eq!(byte_count, length);
            assert_eq!(parser.get_state(), state);
            true
        },
        _ => false
    });
}

pub fn assert_eof<T: HttpHandler + ParamHandler>(parser: &mut Parser<T>, handler: &mut T,
                                                 stream: &[u8], state: State, length: usize) {
    assert!(match parser.parse(handler, stream) {
        Ok(Success::Eof(byte_count)) => {
            assert_eq!(byte_count, length);
            assert_eq!(parser.get_state(), state);
            true
        },
        _ => false
    });
}

pub fn assert_error<T: HttpHandler + ParamHandler>(parser: &mut Parser<T>, handler: &mut T,
                                                   stream: &[u8])
-> Option<ParserError> {
    match parser.parse(handler, stream) {
        Err(error) => {
            assert_eq!(parser.get_state(), State::Dead);
            return Some(error);
        },
        _ => {
            assert_eq!(parser.get_state(), State::Dead);
            None
        }
    }
}

pub fn loop_control<F>(skip: &[u8], function: F) where F : Fn(u8) {
    'outer:
    for n1 in 0..255 {
        for n2 in skip {
            if n1 == *n2 {
                continue 'outer;
            }
        }

        if is_control!(n1) {
            function(n1 as u8);
        }
    }
}

pub fn loop_non_control<F>(skip: &[u8], function: F) where F : Fn(u8) {
    'outer:
    for n1 in 0..255 {
        for n2 in skip {
            if n1 == *n2 {
                continue 'outer;
            }
        }

        if !is_control!(n1) {
            function(n1 as u8);
        }
    }
}

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

pub fn loop_safe<F>(skip: &[u8], function: F) where F : Fn(u8) {
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

pub fn loop_non_safe<F>(skip: &[u8], function: F) where F : Fn(u8) {
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

pub fn setup<T:HttpHandler + ParamHandler>(parser: &mut Parser<T>, handler: &mut T, stream: &[u8],
                                           state: State) {
    assert!(match parser.parse(handler, stream) {
        Ok(Success::Eof(length)) => {
            assert_eq!(length, stream.len());
            assert_eq!(parser.get_state(), state);
            true
        },
        _ => false
    });
}

pub fn vec_eq<T: Debug + PartialEq>(vec: Vec<T>, slice: &[T]) {
    assert_eq!(vec.len(), slice.len());

    for n in 0..vec.len() {
        assert_eq!(vec[n], slice[n]);
    }
}
