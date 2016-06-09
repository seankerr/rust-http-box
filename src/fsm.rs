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

//! Finite state machine macros.

use std::fmt;

// Execute callback `$function`. If it returns `true`, execute `$exec`. Otherwise exit with
// `Success::Callback`.
macro_rules! callback {
    ($parser:expr, $context:expr, $function:ident, $data:expr, $exec:expr) => ({
        if $context.handler.$function($data) {
            $exec
        } else {
            exit_callback!($parser, $context);
        }
    });

    ($parser:expr, $context:expr, $function:ident, $exec:expr) => ({
        let slice = bs_slice!($context);

        if slice.len() > 0 {
            if $context.handler.$function(slice) {
                $exec
            } else {
                exit_callback!($parser, $context);
            }
        } else {
            $exec
        }
    });
}

// Reusable callback EOS expression that executes `$function`.
macro_rules! callback_eos_expr {
    ($parser:expr, $context:expr, $function:ident) => ({
        callback!($parser, $context, $function, {
            exit_eos!($parser, $context);
        });
    });
}

// Execute callback `$function` ignoring the last collected byte. If it returns `true`, transition
// to `$state`. Otherwise exit with `Success::Callback`.
macro_rules! callback_ignore_transition {
    ($parser:expr, $context:expr, $function:ident, $state:expr, $state_function:ident) => ({
        let slice = bs_slice_ignore!($context);

        set_state!($parser, $state, $state_function);

        if slice.len() > 0 {
            if $context.handler.$function(slice) {
                transition!($parser, $context);
            } else {
                exit_callback!($parser, $context);
            }
        } else {
            transition!($parser, $context);
        }
    });
}

// Execute callback `$function` ignoring the last collected byte. If it returns `true`, transition
// to the next `$state` quickly by directly calling `$state_function`. Otherwise exit with
// `Success::Callback`.
macro_rules! callback_ignore_transition_fast {
    ($parser:expr, $context:expr, $function:ident, $state:expr, $state_function:ident) => ({
        let slice = bs_slice_ignore!($context);

        set_state!($parser, $state, $state_function);

        if slice.len() > 0 {
            if $context.handler.$function(slice) {
                transition_fast!($parser, $context);
            } else {
                exit_callback!($parser, $context);
            }
        } else {
            transition_fast!($parser, $context);
        }
    });
}

// Execute callback `$function`. If it returns `true`, transition to the `$state`. Otherwise exit
// with `Success::Callback`.
//
// This macro exists to enforce the design decision that after each callback, state must either
// change, or the parser must exit with `Success::Callback`.
macro_rules! callback_transition {
    ($parser:expr, $context:expr, $function:ident, $data:expr, $state:expr,
     $state_function:ident) => ({
        set_state!($parser, $state, $state_function);
        callback!($parser, $context, $function, $data, {
            transition!($parser, $context);
        });
    });

    ($parser:expr, $context:expr, $function:ident, $state:expr, $state_function:ident) => ({
        set_state!($parser, $state, $state_function);
        callback!($parser, $context, $function, {
            transition!($parser, $context);
        });
    });
}

// Execute callback `$function`. If it returns `true`, transition to the `$state` quickly by
// directly calling `$state_function`. Otherwise exit with `Success::Callback`.
//
// This macro exists to enforce the design decision that after each callback, state must either
// change, or the parser must exit with `Success::Callback`.
macro_rules! callback_transition_fast {
    ($parser:expr, $context:expr, $function:ident, $data:expr, $state:expr,
     $state_function:ident) => ({
        set_state!($parser, $state, $state_function);
        callback!($parser, $context, $function, $data, {
            transition_fast!($parser, $context);
        });
    });

    ($parser:expr, $context:expr, $function:ident, $state:expr, $state_function:ident) => ({
        set_state!($parser, $state, $state_function);
        callback!($parser, $context, $function, {
            transition_fast!($parser, $context);
        });
    });
}

// Collect header value.
macro_rules! collect_header_value {
    ($context:expr, $error:expr, $eos:expr) => ({
        bs_collect!($context, {
                if $context.byte == b'\r' {
                    break;
                } else if $context.byte > 0x1F && $context.byte < 0x7F {
                    // space + visible
                    continue;
                }

                return Err($error($context.byte));
            },
            $eos
        );
    });
}

// Exit parser with `Success::Callback`.
macro_rules! exit_callback {
    ($parser:expr, $context:expr) => ({
        return Ok(ParserValue::Exit(Success::Callback($context.stream_index)));
    });
}

// Exit parser with `Success::Eos`.
macro_rules! exit_eos {
    ($parser:expr, $context:expr) => ({
        return Ok(ParserValue::Exit(Success::Eos($context.stream_index)));
    });
}

// Exit parser with `Success::Finished`.
macro_rules! exit_finished {
    ($parser:expr, $context:expr) => ({
        return Ok(ParserValue::Exit(Success::Finished($context.stream_index)));
    });
}

// If the stream is EOS, exit with `Success::Eos`. Otherwise do nothing.
macro_rules! exit_if_eos {
    ($parser:expr, $context:expr) => ({
        if bs_is_eos!($context) {
            exit_eos!($parser, $context);
        }
    });
}

// Set state and state function.
macro_rules! set_state {
    ($parser:expr, $state:expr, $state_function:ident) => ({
        $parser.state          = $state;
        $parser.state_function = Parser::$state_function;
    });
}

// Transition to `$state`.
macro_rules! transition {
    ($parser:expr, $context:expr, $state:expr, $state_function:ident) => ({
        set_state!($parser, $state, $state_function);

        bs_mark!($context, $context.stream_index);

        return Ok(ParserValue::Continue);
    });

    ($parser:expr, $context:expr) => ({
        $context.mark_index = $context.stream_index;

        return Ok(ParserValue::Continue);
    });
}

// Transition to `$state` quickly by directly calling `$state_function`.
macro_rules! transition_fast {
    ($parser:expr, $context:expr, $state:expr, $state_function:ident) => ({
        set_state!($parser, $state, $state_function);

        bs_mark!($context, $context.stream_index);

        return ($parser.state_function)($parser, $context);
    });

    ($parser:expr, $context:expr) => ({
        $context.mark_index = $context.stream_index;

        return ($parser.state_function)($parser, $context);
    });
}

// -------------------------------------------------------------------------------------------------

/// Parser return values.
pub enum ParserValue {
    /// Continue the parser loop.
    Continue,

    /// Exit the parser loop.
    Exit(Success)
}

// -------------------------------------------------------------------------------------------------

/// Parser success return values.
#[derive(Clone,Copy,PartialEq)]
pub enum Success {
    /// Callback returned `false`.
    ///
    /// The `usize` argument is the amount of `stream` bytes that were processed.
    Callback(usize),

    /// Additional data expected.
    ///
    /// The `usize` argument is the amount of `stream` bytes that were processed.
    Eos(usize),

    /// Finished successfully.
    ///
    /// The `usize` argument is the amount of `stream` bytes that were processed.
    Finished(usize)
}

impl fmt::Debug for Success {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Success::Callback(length) => {
                write!(formatter, "Success::Callback({})", length)
            },
            Success::Eos(length) => {
                write!(formatter, "Success::Eos({})", length)
            },
            Success::Finished(length) => {
                write!(formatter, "Success::Finished({})", length)
            }
        }
    }
}

impl fmt::Display for Success {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Success::Callback(length) => {
                write!(formatter, "{}", length)
            },
            Success::Eos(length) => {
                write!(formatter, "{}", length)
            },
            Success::Finished(length) => {
                write!(formatter, "{}", length)
            }
        }
    }
}