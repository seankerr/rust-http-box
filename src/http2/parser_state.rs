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

//! HTTP 2.x parser states.

/// Parser states.
#[derive(Clone,Copy,Debug,PartialEq)]
#[repr(u8)]
pub enum ParserState {
    /// An error was returned from a call to `Parser::parse()`.
    Dead,

    // ---------------------------------------------------------------------------------------------
    // FRAME STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing frame length first byte.
    FrameLength1,

    /// Parsing frame length second byte.
    FrameLength2,

    /// Parsing frame length third byte.
    FrameLength3,

    /// Parsing frame type.
    FrameType,

    /// Parsing frame flags.
    FrameFlags,

    /// Parsing frame stream identifier first byte.
    FrameStreamId1,

    /// Parsing frame stream identifier second byte.
    FrameStreamId2,

    /// Parsing frame stream identifier third byte.
    FrameStreamId3,

    /// Parsing frame stream identifier fourth byte.
    FrameStreamId4,

    /// Frame format parsing finished.
    FrameFormatEnd,

    /// Parsing end-of-frame padding.
    FramePadding,

    // ---------------------------------------------------------------------------------------------
    // DATA STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing data pad length.
    DataPadLength,

    /// Parsing data.
    DataData,

    // ---------------------------------------------------------------------------------------------
    // GO AWAY STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing go away stream identifier first byte.
    GoAwayStreamId1,

    /// Parsing go away stream identifier second byte.
    GoAwayStreamId2,

    /// Parsing go away stream identifier third byte.
    GoAwayStreamId3,

    /// Parsing go away stream identifier fourth byte.
    GoAwayStreamId4,

    /// Parsing go away error code first byte.
    GoAwayErrorCode1,

    /// Parsing go away error code second byte.
    GoAwayErrorCode2,

    /// Parsing go away error code third byte.
    GoAwayErrorCode3,

    /// Parsing go away error code fourth byte.
    GoAwayErrorCode4,

    /// Executing go away callback.
    GoAwayCallback,

    /// Parsing go away debug data.
    GoAwayDebugData,

    // ---------------------------------------------------------------------------------------------
    // HEADERS STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing headers pad length with priority flag.
    HeadersPadLengthWithPriority,

    /// Parsing headers pad length without priority flag.
    HeadersPadLengthWithoutPriority,

    /// Parsing headers stream identifier first byte.
    HeadersStreamId1,

    /// Parsing headers stream identifier second byte.
    HeadersStreamId2,

    /// Parsing headers stream identifier third byte.
    HeadersStreamId3,

    /// Parsing headers stream identifier fourth byte.
    HeadersStreamId4,

    /// Parsing headers weight.
    HeadersWeight,

    /// Executing headers callback.
    HeadersCallback,

    /// Parsing headers fragment.
    HeadersFragment,

    // ---------------------------------------------------------------------------------------------
    // PING STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing ping data.
    PingData,

    // ---------------------------------------------------------------------------------------------
    // PRIORITY STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing priority stream identifier first byte.
    PriorityStreamId1,

    /// Parsing priority stream identifier second byte.
    PriorityStreamId2,

    /// Parsing priority stream identifier third byte.
    PriorityStreamId3,

    /// Parsing priority stream identifier fourth byte.
    PriorityStreamId4,

    /// Parsing priority weight.
    PriorityWeight,

    // ---------------------------------------------------------------------------------------------
    // PUSH PROMISE STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing push promise pad length.
    PushPromisePadLength,

    /// Parsing push promise stream identifier first byte.
    PushPromiseStreamId1,

    /// Parsing push promise stream identifier second byte.
    PushPromiseStreamId2,

    /// Parsing push promise stream identifier third byte.
    PushPromiseStreamId3,

    /// Parsing push promise stream identifier fourth byte.
    PushPromiseStreamId4,

    /// Executing the push promise callback.
    PushPromiseCallback,

    // ---------------------------------------------------------------------------------------------
    // RST STREAM STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing rst stream error code first byte.
    RstStreamErrorCode1,

    /// Parsing rst stream error code second byte.
    RstStreamErrorCode2,

    /// Parsing rst stream error code third byte.
    RstStreamErrorCode3,

    /// Parsing rst stream error code fourth byte.
    RstStreamErrorCode4,

    /// Executing rst stream callback.
    RstStreamCallback,

    // ---------------------------------------------------------------------------------------------
    // SETTINGS STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing settings identifier first byte.
    SettingsId1,

    /// Parsing settings identifier second byte.
    SettingsId2,

    /// Parsing settings value first byte.
    SettingsValue1,

    /// Parsing settings value second byte.
    SettingsValue2,

    /// Parsing settings value third byte.
    SettingsValue3,

    /// Parsing settings value fourth byte.
    SettingsValue4,

    /// Executing settings callback.
    SettingsCallback,

    // ---------------------------------------------------------------------------------------------
    // UNSUPPORTED STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing unsupported pad length.
    UnsupportedPadLength,

    /// Parsing unsupported data.
    UnsupportedData,

    // ---------------------------------------------------------------------------------------------
    // WINDOW UPDATE STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing window update increment first byte.
    WindowUpdateIncrement1,

    /// Parsing window update increment second byte.
    WindowUpdateIncrement2,

    /// Parsing window update increment third byte.
    WindowUpdateIncrement3,

    /// Parsing window update increment fourth byte.
    WindowUpdateIncrement4,

    /// Executing window update callback.
    WindowUpdateCallback,

    // ---------------------------------------------------------------------------------------------
    // FINISHED
    // ---------------------------------------------------------------------------------------------

    /// Parsing entire message has finished.
    Finished
}
