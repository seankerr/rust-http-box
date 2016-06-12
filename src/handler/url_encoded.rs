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

//! [`Http1Handler`](../http1/trait.Http1Handler.html) implementations for callbacks that deal with
//! URL encoded data.

use http1::Http1Handler;

use std::collections::HashMap;

pub struct MapUrlEncodedHandler<'a> {
    field: Vec<u8>,
    map:   &'a mut HashMap<Vec<u8>, Vec<u8>>,
    value: Vec<u8>
}
/*
impl<'a> MapUrlEncodedHandler<'a> {
    pub fn new(map: &mut HashMap<Vec<u8>, Vec<u8>>) -> MapUrlEncodedHandler<'a> {
        MapUrlEncodedHandler{ field: Vec::new(),
                              map:   &mut map,
                              value: Vec::new() }
    }
}
*/
