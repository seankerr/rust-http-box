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

use byte::{ is_header_field, is_quoted_header_field, is_token };

pub fn alpha_lower_vec() -> Vec<u8> {
    (0x61..0x7B).collect::<Vec<u8>>()
}

pub fn alpha_upper_vec() -> Vec<u8> {
    (0x41..0x5B).collect::<Vec<u8>>()
}

pub fn control_vec() -> Vec<u8> {
    (0..128).filter(|&x| x < 0x20 || x == 0x7F).collect::<Vec<u8>>()
}

pub fn digit_vec() -> Vec<u8> {
    (0x30..0x3A).collect::<Vec<u8>>()
}

pub fn header_field_vec() -> Vec<u8> {
    (0..128).filter(|&x| is_header_field(x)).collect::<Vec<u8>>()
}

pub fn non_control_vec() -> Vec<u8> {
    (0..255).filter(|&x| x > 0x1F && x != 0x7F).collect::<Vec<u8>>()
}

pub fn non_digit_vec() -> Vec<u8> {
    (0..255).filter(|&x| !is_digit!(x)).collect::<Vec<u8>>()
}

pub fn non_token_vec() -> Vec<u8> {
    (0..255).filter(|&x| !is_token(x))
            .collect::<Vec<u8>>()
}

pub fn non_visible_7bit_vec() -> Vec<u8> {
    (0..255).filter(|&x| is_not_visible_7bit!(x)).collect::<Vec<u8>>()
}

pub fn quoted_header_field_vec() -> Vec<u8> {
    (0..255).filter(|&x| is_quoted_header_field(x)).collect::<Vec<u8>>()
}

pub fn token_vec() -> Vec<u8> {
    (0..128).filter(|&x| is_token(x))
            .collect::<Vec<u8>>()
}

pub fn visible_7bit_vec() -> Vec<u8> {
    (0..128).filter(|&x| is_visible_7bit!(x)).collect::<Vec<u8>>()
}
