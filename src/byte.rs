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

//! Byte verification functions.

/// Bytes allowed in non-quoted header fields.
static HEADER_FIELDS: [bool; 255] = [

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

// !   "      #      $      %      &      '     (      )      *
true,  false, true,  true,  true,  true,  true, true,  true,  true,

// +   ,      -      .      /
true,  true,  true,  true,  true,

// 0   1      2      3      4      5      6      7      8      9
true,  true,  true,  true,  true,  true,  true,  true,  true,  true,

// :   ;      <      =      >      ?      @
false, true,  false, true,  false, false, false,

// A   B      C      D      E      F      G      H      I      J
true,  true,  true,  true,  true,  true,  true,  true,  true,  true,

// K   L      M      N      O      P      Q      R      S      T
true,  true,  true,  true,  true,  true,  true,  true,  true,  true,

// U   V      W      X      Y      Z
true,  true,  true,  true,  true,  true,

// [   \      ]      ^      _      `
false, false, false, true,  true,  true,

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
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true

];

/// Bytes allowed in quoted header fields.
static QUOTED_HEADER_FIELDS: [bool; 255] = [

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

// !   "      #      $      %      &      '     (      )      *
true,  false, true,  true,  true,  true,  true,  true,  true,  true,

// +   ,      -      .      /
true,  true, true,  true,  true,

// 0   1      2      3      4      5      6      7      8      9
true,  true, true,  true,  true,  true,  true,  true,  true,  true,

// :   ;      <      =      >      ?      @
true,  true, true,  true,  true,  true,  true,

// A   B      C      D      E      F      G      H      I      J
true,  true, true,  true,  true,  true,  true,  true,  true,  true,

// K   L      M      N      O      P      Q      R      S      T
true,  true, true,  true,  true,  true,  true,  true,  true,  true,

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
true,  true, true,  true,

// DEL
false,

// 128 - 255
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true,  true,  true,  true,
true,  true, true,  true,  true,  true,  true

];

/// Bytes that are considered tokens.
static TOKENS: [bool; 255] = [

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
false, false, false, true,  true,  true,

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
false, false, false, false, false, false, false

];

/// Bytes considered as separators when used in a URL encoded value.
static URL_ENCODED_SEPARATORS: [bool; 255] = [

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
false, false, false, false, true,  true,  false, false, false, false,

// +   ,      -      .      /
true, false,  false, false, false,

// 0   1      2      3      4      5      6      7      8      9
false, false, false, false, false, false, false, false, false, false,

// :   ;      <      =      >      ?      @
false, true,  false, true,  false, false, false,

// A   B      C      D      E      F      G      H      I      J
false, false, false, false, false, false, false, false, false, false,

// K   L      M      N      O      P      Q      R      S      T
false, false, false, false, false, false, false, false, false, false,

// U   V      W      X      Y      Z
false, false, false, false, false, false,

// [   \      ]      ^      _      `
false, false, false, false, false, false,

// a   b      c      d      e      f      g      h      i      j
false, false, false, false, false, false, false, false, false, false,

// k   l      m      n      o      p      q      r      s      t
false, false, false, false, false, false, false, false, false, false,

// u   v      w      x      y      z
false, false, false, false, false, false,

// {   |      }      ~
false, false, false, false,

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
false, false, false, false, false, false, false

];

/// Indicates that a byte is allowed in non-quoted header field.
#[inline]
pub fn is_header_field(byte: u8) -> bool {
    HEADER_FIELDS[byte as usize]
}

/// Indicates that a byte is allowed in a quoted header field.
///
/// This excludes `"` and `\`, so that a collection loop will break.
#[inline]
pub fn is_quoted_header_field(byte: u8) -> bool {
    QUOTED_HEADER_FIELDS[byte as usize]
}

/// Indicates that a byte is a HTTP token.
#[inline]
pub fn is_token(byte: u8) -> bool {
    TOKENS[byte as usize]
}

/// Indicates that a byte is a URL encoded separator.
#[inline]
pub fn is_url_encoded_separator(byte: u8) -> bool {
    URL_ENCODED_SEPARATORS[byte as usize]
}
