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

//! Finite state machine macros and types.

use std::fmt;

/// Execute callback `$callback`. If it returns `true`, execute `$exec`. Otherwise exit with
/// `Success::Callback`.
macro_rules! callback {
    ($parser:expr, $handler:expr, $context:expr, $callback:ident, $data:expr, $exec:expr) => ({
        if $handler.$callback($data) {
            $exec
        }

        exit_callback!($parser, $context);
    });

    ($parser:expr, $handler:expr, $context:expr, $callback:ident, $exec:expr) => ({
        if bs_slice_length!($context) > 0 {
            if $handler.$callback(bs_slice!($context)) {
                $exec
            }

            exit_callback!($parser, $context);
        }

        $exec
    });
}

/// Reusable callback EOS expression that executes `$callback`.
macro_rules! callback_eos_expr {
    ($parser:expr, $handler:expr, $context:expr, $callback:ident) => ({
        callback!($parser, $handler, $context, $callback, {
            exit_eos!($parser, $context);
        });
    });
}

/// Execute callback `$callback` ignoring the last collected byte. If it returns `true`, transition
/// to `$state`. Otherwise exit with `Success::Callback`.
macro_rules! callback_ignore_transition {
    ($parser:expr, $handler:expr, $context:expr, $callback:ident, $state:ident,
     $state_function:ident) => ({
        set_state!($parser, $state, $state_function);

        // compare against 1 instead of 0 because we're ignoring the last slice byte
        if bs_slice_length!($context) > 1 {
            if $handler.$callback(bs_slice_ignore!($context)) {
                transition!($parser, $context);
            }

            exit_callback!($parser, $context);
        }

        transition!($parser, $context);
    });
}

/// Execute callback `$callback` ignoring the last collected byte. If it returns `true`, transition
/// to the next `$state` quickly by directly calling `$state_function`. Otherwise exit with
/// `Success::Callback`.
macro_rules! callback_ignore_transition_fast {
    ($parser:expr, $handler:expr, $context:expr, $callback:ident, $state:ident,
     $state_function:ident) => ({
         set_state!($parser, $state, $state_function);

         // compare against 1 instead of 0 because we're ignoring the last slice byte
         if bs_slice_length!($context) > 1 {
             if $handler.$callback(bs_slice_ignore!($context)) {
                 transition_fast!($parser, $handler, $context);
             }

             exit_callback!($parser, $context);
         }

         transition_fast!($parser, $handler, $context);
    });
}

/// Execute callback `$callback`. If it returns `true`, transition to `$state`. Otherwise exit
/// with `Success::Callback`.
///
/// This macro exists to enforce the design decision that after each callback, state must either
/// change, or the parser must exit with `Success::Callback`.
macro_rules! callback_transition {
    ($parser:expr, $handler:expr, $context:expr, $callback:ident, $data:expr, $state:ident,
     $state_function:ident) => ({
        set_state!($parser, $state, $state_function);
        callback!($parser, $handler, $context, $callback, $data, {
            transition!($parser, $context);
        });
    });

    ($parser:expr, $handler:expr, $context:expr, $callback:ident, $state:ident,
     $state_function:ident) => ({
        set_state!($parser, $state, $state_function);
        callback!($parser, $handler, $context, $callback, {
            transition!($parser, $context);
        });
    });
}

/// Execute callback `$callback`. If it returns `true`, transition to `$state` quickly by
/// directly calling `$state_function`. Otherwise exit with `Success::Callback`.
///
/// This macro exists to enforce the design decision that after each callback, state must either
/// change, or the parser must exit with `Success::Callback`.
macro_rules! callback_transition_fast {
    ($parser:expr, $handler:expr, $context:expr, $callback:ident, $data:expr, $state:ident,
     $state_function:ident) => ({
        set_state!($parser, $state, $state_function);
        callback!($parser, $handler, $context, $callback, $data, {
            transition_fast!($parser, $handler, $context);
        });
    });

    ($parser:expr, $handler:expr, $context:expr, $callback:ident, $state:ident,
     $state_function:ident) => ({
        set_state!($parser, $state, $state_function);
        callback!($parser, $handler, $context, $callback, {
            transition_fast!($parser, $handler, $context);
        });
    });
}

/// Exit parser with `Success::Callback`.
macro_rules! exit_callback {
    ($parser:expr, $context:expr, $state:ident, $state_function:ident) => ({
        set_state!($parser, $state, $state_function);

        return Ok(ParserValue::Exit(Success::Callback($context.stream_index)));
    });

    ($parser:expr, $context:expr) => ({
        return Ok(ParserValue::Exit(Success::Callback($context.stream_index)));
    });
}

/// Exit parser with `Success::Eos`.
macro_rules! exit_eos {
    ($parser:expr, $context:expr) => ({
        return Ok(ParserValue::Exit(Success::Eos($context.stream_index)));
    });
}

/// Exit parser with `ParserError`.
macro_rules! exit_error {
    ($error:ident, $byte:expr) => ({
        return Err(ParserError::$error($byte));
    });

    ($error:ident) => ({
        return Err(ParserError::$error);
    });
}

/// Exit parser with `Success::Finished`.
macro_rules! exit_finished {
    ($parser:expr, $context:expr) => ({
        return Ok(ParserValue::Exit(Success::Finished($context.stream_index)));
    });
}

/// If the stream is EOS, exit with `Success::Eos`. Otherwise do nothing.
macro_rules! exit_if_eos {
    ($parser:expr, $context:expr) => ({
        bs_available!($context) > 0 || exit_eos!($parser, $context);
    });
}

/// Retrieve the state.
macro_rules! get_state {
    ($parser:expr) => ({
        $parser.state
    })
}

/// Set state and state function.
macro_rules! set_state {
    ($parser:expr, $state:ident, $state_function:ident) => ({
        $parser.state          = ParserState::$state;
        $parser.state_function = Parser::$state_function;
    });
}

/// Transition to `$state`.
macro_rules! transition {
    ($parser:expr, $context:expr, $state:ident, $state_function:ident) => ({
        set_state!($parser, $state, $state_function);

        bs_mark!($context, $context.stream_index);

        return Ok(ParserValue::Continue);
    });

    ($parser:expr, $context:expr) => ({
        bs_mark!($context, $context.stream_index);

        return Ok(ParserValue::Continue);
    });
}

/// Transition to `$state` quickly by directly calling `$state_function`.
macro_rules! transition_fast {
    ($parser:expr, $handler:expr, $context:expr, $state:ident, $state_function:ident) => ({
        set_state!($parser, $state, $state_function);

        bs_mark!($context, $context.stream_index);

        return ($parser.state_function)($parser, $handler, $context);
    });

    ($parser:expr, $handler:expr, $context:expr) => ({
        bs_mark!($context, $context.stream_index);

        return ($parser.state_function)($parser, $handler, $context);
    });
}

/// Transition to `$state` quickly by directly calling `$state_function`.
///
/// This will not readjust the mark index.
macro_rules! transition_fast_no_remark {
    ($parser:expr, $handler:expr, $context:expr, $state:ident, $state_function:ident) => ({
        set_state!($parser, $state, $state_function);

        return ($parser.state_function)($parser, $handler, $context);
    });
}

/// Transition to `$state`.
///
/// This will not readjust the mark index.
macro_rules! transition_no_remark {
    ($parser:expr, $context:expr, $state:ident, $state_function:ident) => ({
        set_state!($parser, $state, $state_function);

        return Ok(ParserValue::Continue);
    });
}

// -------------------------------------------------------------------------------------------------

/// Parsing function return values.
pub enum ParserValue {
    /// Continue the parser loop.
    Continue,

    /// Exit the parser loop.
    Exit(Success)
}

// -------------------------------------------------------------------------------------------------

/// Parsing function success return values.
#[derive(Clone,Copy,PartialEq)]
pub enum Success {
    /// A callback returned `false` and the parser function exited prematurely. This can be
    /// treated the same as `Success::Finished`.
    ///
    /// # Arguments
    ///
    /// **(1)**: The amount of `stream` bytes that were processed before the callback was executed.
    ///          In most cases this will not match `stream.len()`.
    Callback(usize),

    /// Additional `stream` data is expected. Continue executing the parser function until
    /// `Success::Finished` is returned.
    ///
    /// # Arguments
    ///
    /// **(1)**: The amount of `stream` bytes that were processed. This value will always match
    ///          `stream.len()`.
    Eos(usize),

    /// The parser function finished successfully.
    ///
    /// # Arguments
    ///
    /// **(1)**: The amount of `stream` bytes that were processed. Under some circumstances this
    ///          will be less than `stream.len()`. This indicates that there must be a transition
    ///          between the current parser function and the next one. For example, a typical HTTP
    ///          request would consist of a call to
    ///          [Parser::parse_head()](../http1/struct.Parser.html#method.parse_head), and
    ///          depending on the content type you may need to transition to
    ///          [Parser::parse_chunked()](../http1/struct.Parser.html#method.parse_chunked),
    ///          [Parser::parse_multipart()](../http1/struct.Parser.html#method.parse_multipart), or
    ///          [Parser::parse_url_encoded()](../http1/struct.Parser.html#method.parse_url_encoded).
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
