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

//! Byte verification functions.

/// Indicates that a byte should be encoded to hex.
///
/// This encodes all control characters `0x00` thru `0x1F`, DEL `0x7F`, SPC `0x20`, and all
/// ISO-Latin characters `0x80` thru `0xFF`.
///
/// This follows the list of delimiters listed in RFC 3986, as well as additional characters known
/// to be used by HTTP and HTML parsers that for safety reasons are also included.
///
/// *General delimiters:*
///
/// `:`, `/`, `?`, `#`, `[`, `]`, `@`
///
/// *Sub-delimiters:*
///
/// `!`, `$`, `&`, `'`, `(`, `)`, `*`, `+`, `,`, `;`, `=`
///
/// *Additional characters:*
///
/// `<`, `>`, `\`, `^`, `` ` ``, `{`, `}`, `|`
#[inline]
pub fn is_encoded(byte: u8) -> bool {
    [

    // NUL SOH    STX    ETX    EOT    ENQ    ACK    BEL    BS     TAB
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,

    // LF  VT     FF     CR     SO     SI     DLE    DC1    DC2    DC3
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,

    // DC4 NAK    SYN    ETB    CAN    EM     SUB    ESC    FS     GS
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,

    // RS  US
    true,  true,

    // space
    true,

    // !   "      #      $      %      &      '      (      )      *
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,

    // +   ,      -      .      /
    true,  true,  false, false, true,

    // 0   1      2      3      4      5      6      7      8      9
    false, false, false, false, false, false, false, false, false, false,

    // :   ;      <      =      >      ?      @
    true,  true,  true,  true,  true,  true,  true,

    // A   B      C      D      E      F      G      H      I      J
    false, false, false, false, false, false, false, false, false, false,

    // K   L      M      N      O      P      Q      R      S      T
    false, false, false, false, false, false, false, false, false, false,

    // U   V      W      X      Y      Z
    false, false, false, false, false, false,

    // [   \      ]      ^      _      `
    true,  true,  true,  true,  false, true,

    // a   b      c      d      e      f      g      h      i      j
    false, false, false, false, false, false, false, false, false, false,

    // k   l      m      n      o      p      q      r      s      t
    false, false, false, false, false, false, false, false, false, false,

    // u   v      w      x      y      z
    false, false, false, false, false, false,

    // {   |      }      ~
    true,  true,  true,  false,

    // DEL
    true,

    // 128 - 255
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,
    true,  true,  true,  true,  true,  true,  true,  true

    ][byte as usize]
}

/// Indicates that a byte is a HTTP separator.
#[inline]
pub fn is_separator(byte: u8) -> bool {
    [

    // NUL SOH    STX    ETX    EOT    ENQ    ACK    BEL    BS     TAB
    false, false, false, false, false, false, false, false, false, true,

    // LF  VT     FF     CR     SO     SI     DLE    DC1    DC2    DC3
    false, false, false, false, false, false, false, false, false, false,

    // DC4 NAK    SYN    ETB    CAN    EM     SUB    ESC    FS     GS
    false, false, false, false, false, false, false, false, false, false,

    // RS  US
    false, false,

    // space
    true,

    // !   "      #      $      %      &      '      (      )      *
    false, true,  false, false, false, false, false, true,  true,  false,

    // +   ,      -      .      /
    false, true, false, false, true,

    // 0   1      2      3      4      5      6      7      8      9
    false, false, false, false, false, false, false, false, false, false,

    // :   ;      <      =      >      ?      @
    true,  true,  true,  true,  true,  true,  true,

    // A   B      C      D      E      F      G      H      I      J
    false, false, false, false, false, false, false, false, false, false,

    // K   L      M      N      O      P      Q      R      S      T
    false, false, false, false, false, false, false, false, false, false,

    // U   V      W      X      Y      Z
    false, false, false, false, false, false,

    // [   \      ]      ^      _      `
    true,  true,  true,  false, false, false,

    // a   b      c      d      e      f      g      h      i      j
    false, false, false, false, false, false, false, false, false, false,

    // k   l      m      n      o      p      q      r      s      t
    false, false, false, false, false, false, false, false, false, false,

    // u   v      w      x      y      z
    false, false, false, false, false, false,

    // {   |      }      ~
    true,  false, true,  false,

    // DEL
    false,

    // 128 - 255
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false

    ][byte as usize]
}

/// Indicates that a byte is a HTTP token.
#[inline]
pub fn is_token(byte: u8) -> bool {
    [

    // NUL SOH    STX    ETX    EOT    ENQ    ACK    BEL    BS     TAB
    false, false, false, false, false, false, false, false, false, false,

    // LF  VT     FF     CR     SO     SI     DLE    DC1    DC2    DC3
    false, false, false, false, false, false, false, false, false, false,

    // DC4 NAK    SYN    ETB    CAN    EM     SUB    ESC    FS     GS
    false, false, false, false, false, false, false, false, false, false,

    // RS  US
    false, false,

    // space
    false,

    // !   "      #      $      %      &      '      (      )      *
    true,  false, true,  true,  true,  true,  true,  false, false, true,

    // +   ,      -      .      /
    true,  false, true,  true,  false,

    // 0   1      2      3      4      5      6      7      8      9
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,

    // :   ;      <      =      >      ?      @
    false, false, false, false, false, false, false,

    // A   B      C      D      E      F      G      H      I      J
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,

    // K   L      M      N      O      P      Q      R      S      T
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,

    // U   V      W      X      Y      Z
    true,  true,  true,  true,  true,  true,

    // [   \      ]      ^      _      `
    true,  false, true,  true,  true,  true,

    // a   b      c      d      e      f      g      h      i      j
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,

    // k   l      m      n      o      p      q      r      s      t
    true,  true,  true,  true,  true,  true,  true,  true,  true,  true,

    // u   v      w      x      y      z
    true,  true,  true,  true,  true,  true,

    // {   |      }      ~
    false, true,  false, true,

    // DEL
    false,

    // 128 - 255
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false

    ][byte as usize]
}
