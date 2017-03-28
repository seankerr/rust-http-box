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

use byte_slice::ByteStream;

use std::fmt;

/// Decoding errors.
pub enum DecodeError {
    /// Invalid byte.
    Byte(u8),

    /// Invalid hex sequence.
    HexSequence(u8)
}

impl DecodeError {
    /// Format this for debug and display purposes.
    fn format(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DecodeError::Byte(x) => {
                write!(
                    formatter,
                    "<DecodeError::Byte: {}>",
                    x
                )
            },
            DecodeError::HexSequence(x) => {
                write!(
                    formatter,
                    "<DecodeError::HexSequence: {}>",
                    x
                )
            }
        }
    }
}

impl fmt::Debug for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.format(formatter)
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.format(formatter)
    }
}

// -------------------------------------------------------------------------------------------------

/// Decode URL encoded data.
///
/// # Arguments
///
/// **`encoded`**
///
/// The encoded data.
///
/// # Returns
///
/// **`String`**
///
/// The decoded string.
///
/// # Errors
///
/// - [`DecodeError::Byte`](enum.DecodeError.html#variant.Byte)
/// - [`DecodeError::HexSequence`](enum.DecodeError.html#variant.HexSequence)
///
/// # Examples
///
/// ```
/// use http_box::util;
///
/// let string = match util::decode(b"fancy%20url%20encoded%20data") {
///     Ok(string) => string,
///     Err(_) => panic!()
/// };
///
/// assert_eq!(string, "fancy url encoded data");
/// ```
pub fn decode(encoded: &[u8]) -> Result<String, DecodeError> {
    macro_rules! submit {
        ($string:expr, $slice:expr) => (unsafe {
            $string.as_mut_vec().extend_from_slice($slice);
        });
    }

    let mut context = ByteStream::new(encoded);
    let mut string  = String::new();

    loop {
        bs_mark!(context);

        collect_visible_7bit!(
            context,

            // stop on these bytes
               context.byte == b'+'
            || context.byte == b'%',

            // on end-of-stream
            {
                if context.mark_index < context.stream_index {
                    submit!(string, bs_slice!(context));
                }

                return Ok(string);
            }
        );

        if bs_slice_length!(context) > 1 {
            submit!(string, bs_slice_ignore!(context));
        }

        if context.byte == b'+' {
            submit!(string, b" ");
        } else if context.byte == b'%' {
            if bs_has_bytes!(context, 2) {
                submit!(string, &[
                    collect_hex8!(context, DecodeError::HexSequence)
                ]);
            } else {
                if bs_has_bytes!(context, 1) {
                    bs_next!(context);
                }

                return Err(DecodeError::HexSequence(context.byte));
            }
        } else {
            return Err(DecodeError::Byte(context.byte));
        }
    }
}
