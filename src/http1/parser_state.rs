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

//! HTTP 1.x parser states.

/// Parser states.
#[derive(Clone,Copy,Debug,PartialEq)]
#[repr(u8)]
pub enum ParserState {
    /// An error was returned from a call to `Parser::parse()`.
    Dead,

    /// Stripping linear white space before request/response detection.
    StripDetect,

    /// Detect request/response byte 1.
    Detect1,

    /// Detect request/response byte 2.
    Detect2,

    /// Detect request/response byte 3.
    Detect3,

    /// Detect request/response byte 4.
    Detect4,

    /// Detect request/response byte 5.
    Detect5,

    // ---------------------------------------------------------------------------------------------
    // REQUEST
    // ---------------------------------------------------------------------------------------------

    /// Parsing request method.
    RequestMethod,

    /// Parsing request URL byte 1.
    RequestUrl1,

    /// Parsing request URL byte 2+.
    RequestUrl2,

    /// Parsing request HTTP version byte 1.
    RequestHttp1,

    /// Parsing request HTTP version byte 2.
    RequestHttp2,

    /// Parsing request HTTP version byte 3.
    RequestHttp3,

    /// Parsing request HTTP version byte 4.
    RequestHttp4,

    /// Parsing request HTTP version byte 5.
    RequestHttp5,

    /// Parsing request HTTP major version byte 1.
    RequestVersionMajor1,

    /// Parsing request HTTP major version byte 2.
    RequestVersionMajor2,

    /// Parsing request HTTP major version byte 3.
    RequestVersionMajor3,

    /// Parsing period between HTTP major and minor versions.
    RequestVersionPeriod,

    /// Parsing request HTTP minor version byte 1.
    RequestVersionMinor1,

    /// Parsing request HTTP minor version byte 2.
    RequestVersionMinor2,

    /// Parsing request HTTP minor version byte 3.
    RequestVersionMinor3,

    /// Parsing carriage return after request HTTP minor version.
    RequestVersionCr,

    // ---------------------------------------------------------------------------------------------
    // RESPONSE
    // ---------------------------------------------------------------------------------------------

    /// Parsing response HTTP major version byte 1.
    ResponseVersionMajor1,

    /// Parsing response HTTP major version byte 2.
    ResponseVersionMajor2,

    /// Parsing response HTTP major version byte 3.
    ResponseVersionMajor3,

    /// Parsing period between HTTP major and minor versions.
    ResponseVersionPeriod,

    /// Parsing response HTTP minor version byte 1.
    ResponseVersionMinor1,

    /// Parsing response HTTP minor version byte 2.
    ResponseVersionMinor2,

    /// Parsing response HTTP minor version byte 3.
    ResponseVersionMinor3,

    /// Parsing space after response HTTP minor version.
    ResponseVersionSpace,

    /// Parsing response status code byte 1.
    ResponseStatusCode1,

    /// Parsing response status code byte 2.
    ResponseStatusCode2,

    /// Parsing response status code byte 3.
    ResponseStatusCode3,

    /// Parsing space after response status code.
    ResponseStatusCodeSpace,

    /// Parsing response status byte 1.
    ResponseStatus1,

    /// Parsing response status byte 2+.
    ResponseStatus2,

    // ---------------------------------------------------------------------------------------------
    // HEADERS
    // ---------------------------------------------------------------------------------------------

    /// Parsing initial request/response line has finished.
    InitialEnd,

    /// Parsing line feed after initial request/response line.
    InitialLf,

    /// Checking header name to see if it starts with a space or tab (multiline value).
    CheckHeaderName,

    /// Parsing first byte of header name.
    FirstHeaderName,

    /// Parsing upper-cased header name.
    UpperHeaderName,

    /// Parsing lower-cased header name.
    LowerHeaderName,

    /// Stripping linear white space before header value.
    StripHeaderValue,

    /// Parsing header value.
    HeaderValue,

    /// Parsing quoted header value.
    HeaderQuotedValue,

    /// Parsing escaped header value.
    HeaderEscapedValue,

    /// Parsing first carriage return after status line or header value.
    HeaderCr1,

    /// Parsing first line feed after status line or header value.
    HeaderLf1,

    /// Parsing second carriage return after status line or header value.
    HeaderCr2,

    /// Parsing second line feed after status line or header value.
    HeaderLf2,

    /// Processing end-of-header flag checks.
    HeaderEnd,

    // ---------------------------------------------------------------------------------------------
    // CHUNKED TRANSFER
    // ---------------------------------------------------------------------------------------------

    /// Parsing chunk length byte 1.
    ChunkLength1,

    /// Parsing chunk length byte 2.
    ChunkLength2,

    /// Parsing chunk length byte 3.
    ChunkLength3,

    /// Parsing chunk length byte 4.
    ChunkLength4,

    /// Parsing chunk length byte 5.
    ChunkLength5,

    /// Parsing chunk length byte 6.
    ChunkLength6,

    /// Parsing chunk length byte 7.
    ChunkLength7,

    /// Parsing chunk length byte 8.
    ChunkLength8,

    /// Parsing chunk length carriage return or semi-colon.
    ChunkLengthCr,

    /// Stripping linear white space before chunk extension name.
    StripChunkExtensionName,

    /// Parsing upper-cased chunk extension.
    UpperChunkExtensionName,

    /// Parsing lower-cased chunk extension.
    LowerChunkExtensionName,

    /// Stripping linear white space before chunk extension value.
    StripChunkExtensionValue,

    /// Parsing chunk extension value.
    ChunkExtensionValue,

    /// Parsing quoted chunk extension value.
    ChunkExtensionQuotedValue,

    /// Parsing escaped chunk extension value.
    ChunkExtensionEscapedValue,

    /// End of chunk extension.
    ChunkExtensionFinished,

    /// End of all chunk extensions.
    ChunkExtensionsFinished,

    /// Parsing line feed after chunk length.
    ChunkLengthLf,

    /// Parsing chunk data.
    ChunkData,

    /// Parsing carriage return after chunk data.
    ChunkDataCr,

    /// Parsing line feed after chunk data.
    ChunkDataLf,

    // ---------------------------------------------------------------------------------------------
    // MULTIPART
    // ---------------------------------------------------------------------------------------------

    /// Parsing pre boundary hyphen 1.
    MultipartHyphen1,

    /// Parsing pre boundary hyphen 2.
    MultipartHyphen2,

    /// Parsing multipart boundary.
    MultipartBoundary,

    /// Detecting multipart data parsing mechanism.
    MultipartDetectData,

    /// Parsing multipart data by byte.
    MultipartDataByByte,

    /// Parsing multipart data by content length.
    MultipartDataByLength,

    /// Parsing carriage return after data by length.
    MultipartDataByLengthCr,

    /// Parsing line feed after data by length.
    MultipartDataByLengthLf,

    /// Parsing potential line feed after data by byte.
    MultipartDataByByteLf,

    /// Parsing post boundary carriage return or hyphen.
    MultipartBoundaryCr,

    /// Parsing post boundary line feed.
    MultipartBoundaryLf,

    /// Parsing last boundary second hyphen that indicates end of multipart body.
    MultipartEnd,

    // ---------------------------------------------------------------------------------------------
    // URL ENCODED
    // ---------------------------------------------------------------------------------------------

    /// Parsing first byte of URL encoded name.
    FirstUrlEncodedName,

    /// Parsing URL encoded name.
    UrlEncodedName,

    /// Parsing URL encoded name hex sequence byte 1.
    UrlEncodedNameHex1,

    /// Parsing URL encoded name hex sequence byte 2.
    UrlEncodedNameHex2,

    /// Parsing URL encoded name plus sign.
    UrlEncodedNamePlus,

    /// Parsing URL encoded value.
    UrlEncodedValue,

    /// Parsing URL encoded value hex sequence byte 1.
    UrlEncodedValueHex1,

    /// Parsing URL encoded value hex sequence byte 2.
    UrlEncodedValueHex2,

    /// Parsing URL encoded value plus sign.
    UrlEncodedValuePlus,

    // ---------------------------------------------------------------------------------------------
    // FINISHED
    // ---------------------------------------------------------------------------------------------

    /// End of body parsing.
    BodyFinished,

    /// Parsing entire message has finished.
    Finished
}

// -------------------------------------------------------------------------------------------------

/// State listing in parsing order.
///
/// This is a helper type that will simplify state tracking in custom
/// [HttpHandler](trait.HttpHandler.html) implementations.
#[derive(Clone,Copy,PartialEq)]
#[repr(u8)]
pub enum State {
    None,

    // ---------------------------------------------------------------------------------------------
    // STATUS LINE STATES
    // ---------------------------------------------------------------------------------------------

    /// Request method.
    RequestMethod,

    /// Request URL.
    RequestUrl,

    /// Request HTTP version.
    RequestVersion,

    /// Response HTTP version.
    ResponseVersion,

    /// Response status code.
    ResponseStatusCode,

    /// Response status.
    ResponseStatus,

    // ---------------------------------------------------------------------------------------------
    // HEADER STATES
    // ---------------------------------------------------------------------------------------------

    /// Header name.
    HeaderName,

    /// Header value.
    HeaderValue,

    // ---------------------------------------------------------------------------------------------
    // CHUNK TRANSFER ENCODING STATES
    // ---------------------------------------------------------------------------------------------

    /// Chunk length.
    ChunkLength,

    /// Chunk extension name.
    ChunkExtensionName,

    /// Chunk extension value.
    ChunkExtensionValue,

    /// Chunk data.
    ChunkData,

    // ---------------------------------------------------------------------------------------------
    // MULTIPART STATES
    // ---------------------------------------------------------------------------------------------

    /// Multipart data.
    MultipartData,

    // ---------------------------------------------------------------------------------------------
    // URL ENCODED STATES
    // ---------------------------------------------------------------------------------------------

    /// URL encoded name.
    UrlEncodedName,

    /// URL encoded value.
    UrlEncodedValue
}
