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

use super::{HttpHandler, Parser, ParserError};

use fsm::ParserValue;

use byte_slice::ByteStream;

macro_rules! state_dispatch {
    (
        $(#[$state_attr:meta])*
        pub enum $State:ident {$(
            #[$var_doc:meta] $Var:ident($func:ident)
        ),*}
    ) => {
        $(#[$state_attr])*
        pub enum $State {$(
            #[$var_doc] $Var,
        )*}


        pub fn dispatch<'dispatch, T: HttpHandler>(parser: &mut Parser<'dispatch>,
                                                   handler: &mut T,
                                                   context: &mut ByteStream)
        -> Result<ParserValue, ParserError> {
            match parser.state {$(
                $State::$Var => parser.$func(handler, context),
            )*}
        }
    }
}

state_dispatch! {
    /// Parser states.
    #[derive(Clone,Copy,Debug,PartialEq)]
    #[repr(u8)]
    pub enum ParserState {
        /// An error was returned from a call to `Parser::parse()`.
        Dead(dead),

        /// Stripping linear white space before request/response detection.
        StripDetect(strip_detect),

        /// Detect request/response byte 1.
        Detect1(detect1),

        /// Detect request/response byte 2.
        Detect2(detect2),

        /// Detect request/response byte 3.
        Detect3(detect3),

        /// Detect request/response byte 4.
        Detect4(detect4),

        /// Detect request/response byte 5.
        Detect5(detect5),

        // ---------------------------------------------------------------------------------------------
        // REQUEST
        // ---------------------------------------------------------------------------------------------

        /// Parsing request method.
        RequestMethod(request_method),

        /// Parsing request URL byte 1.
        RequestUrl1(request_url1),

        /// Parsing request URL byte 2+.
        RequestUrl2(request_url2),

        /// Parsing request HTTP version byte 1.
        RequestHttp1(request_http1),

        /// Parsing request HTTP version byte 2.
        RequestHttp2(request_http2),

        /// Parsing request HTTP version byte 3.
        RequestHttp3(request_http3),

        /// Parsing request HTTP version byte 4.
        RequestHttp4(request_http4),

        /// Parsing request HTTP version byte 5.
        RequestHttp5(request_http5),

        /// Parsing request HTTP major version byte 1.
        RequestVersionMajor1(request_version_major1),

        /// Parsing request HTTP major version byte 2.
        RequestVersionMajor2(request_version_major2),

        /// Parsing request HTTP major version byte 3.
        RequestVersionMajor3(request_version_major3),

        /// Parsing period between HTTP major and minor versions.
        RequestVersionPeriod(request_version_period),

        /// Parsing request HTTP minor version byte 1.
        RequestVersionMinor1(request_version_minor1),

        /// Parsing request HTTP minor version byte 2.
        RequestVersionMinor2(request_version_minor2),

        /// Parsing request HTTP minor version byte 3.
        RequestVersionMinor3(request_version_minor3),

        /// Parsing carriage return after request HTTP minor version.
        RequestVersionCr(request_version_cr),

        // ---------------------------------------------------------------------------------------------
        // RESPONSE
        // ---------------------------------------------------------------------------------------------

        /// Parsing response HTTP major version byte 1.
        ResponseVersionMajor1(response_version_major1),

        /// Parsing response HTTP major version byte 2.
        ResponseVersionMajor2(response_version_major2),

        /// Parsing response HTTP major version byte 3.
        ResponseVersionMajor3(response_version_major3),

        /// Parsing period between HTTP major and minor versions.
        ResponseVersionPeriod(response_version_period),

        /// Parsing response HTTP minor version byte 1.
        ResponseVersionMinor1(response_version_minor1),

        /// Parsing response HTTP minor version byte 2.
        ResponseVersionMinor2(response_version_minor2),

        /// Parsing response HTTP minor version byte 3.
        ResponseVersionMinor3(response_version_minor3),

        /// Parsing space after response HTTP minor version.
        ResponseVersionSpace(response_version_space),

        /// Parsing response status code byte 1.
        ResponseStatusCode1(response_status_code1),

        /// Parsing response status code byte 2.
        ResponseStatusCode2(response_status_code2),

        /// Parsing response status code byte 3.
        ResponseStatusCode3(response_status_code3),

        /// Parsing space after response status code.
        ResponseStatusCodeSpace(response_status_code_space),

        /// Parsing response status byte 1.
        ResponseStatus1(response_status1),

        /// Parsing response status byte 2+.
        ResponseStatus2(response_status2),

        // ---------------------------------------------------------------------------------------------
        // HEADERS
        // ---------------------------------------------------------------------------------------------

        /// Parsing initial request/response line has finished.
        InitialEnd(initial_end),

        /// Parsing line feed after initial request/response line.
        InitialLf(initial_lf),

        /// Checking header name to see if it starts with a space or tab (multiline value).
        CheckHeaderName(check_header_name),

        /// Parsing first byte of header name.
        FirstHeaderName(first_header_name),

        /// Parsing upper-cased header name.
        UpperHeaderName(upper_header_name),

        /// Parsing lower-cased header name.
        LowerHeaderName(lower_header_name),

        /// Stripping linear white space before header value.
        StripHeaderValue(strip_header_value),

        /// Parsing header value.
        HeaderValue(header_value),

        /// Parsing quoted header value.
        HeaderQuotedValue(header_quoted_value),

        /// Parsing escaped header value.
        HeaderEscapedValue(header_escaped_value),

        /// Parsing first carriage return after status line or header value.
        HeaderCr1(header_cr1),

        /// Parsing first line feed after status line or header value.
        HeaderLf1(header_lf1),

        /// Parsing second carriage return after status line or header value.
        HeaderCr2(header_cr2),

        /// Parsing second line feed after status line or header value.
        HeaderLf2(header_lf2),

        /// Processing end-of-header flag checks.
        HeaderEnd(header_end),

        // ---------------------------------------------------------------------------------------------
        // CHUNKED TRANSFER
        // ---------------------------------------------------------------------------------------------

        /// Parsing chunk length byte 1.
        ChunkLength1(chunk_length1),

        /// Parsing chunk length byte 2.
        ChunkLength2(chunk_length2),

        /// Parsing chunk length byte 3.
        ChunkLength3(chunk_length3),

        /// Parsing chunk length byte 4.
        ChunkLength4(chunk_length4),

        /// Parsing chunk length byte 5.
        ChunkLength5(chunk_length5),

        /// Parsing chunk length byte 6.
        ChunkLength6(chunk_length6),

        /// Parsing chunk length byte 7.
        ChunkLength7(chunk_length7),

        /// Parsing chunk length byte 8.
        ChunkLength8(chunk_length8),

        /// Parsing chunk length carriage return or semi-colon.
        ChunkLengthCr(chunk_length_cr),

        /// Stripping linear white space before chunk extension name.
        StripChunkExtensionName(strip_chunk_extension_name),

        /// Parsing upper-cased chunk extension.
        UpperChunkExtensionName(upper_chunk_extension_name),

        /// Parsing lower-cased chunk extension.
        LowerChunkExtensionName(lower_chunk_extension_name),

        /// Stripping linear white space before chunk extension value.
        StripChunkExtensionValue(strip_chunk_extension_value),

        /// Parsing chunk extension value.
        ChunkExtensionValue(chunk_extension_value),

        /// Parsing quoted chunk extension value.
        ChunkExtensionQuotedValue(chunk_extension_quoted_value),

        /// Parsing escaped chunk extension value.
        ChunkExtensionEscapedValue(chunk_extension_escaped_value),

        /// End of chunk extension.
        ChunkExtensionFinished(chunk_extension_finished),

        /// End of all chunk extensions.
        ChunkExtensionsFinished(chunk_extensions_finished),

        /// Parsing line feed after chunk length.
        ChunkLengthLf(chunk_length_lf),

        /// Parsing chunk data.
        ChunkData(chunk_data),

        /// Parsing carriage return after chunk data.
        ChunkDataCr(chunk_data_cr),

        /// Parsing line feed after chunk data.
        ChunkDataLf(chunk_data_lf),

        // ---------------------------------------------------------------------------------------------
        // MULTIPART
        // ---------------------------------------------------------------------------------------------

        /// Parsing pre boundary hyphen 1.
        MultipartHyphen1(multipart_hyphen1),

        /// Parsing pre boundary hyphen 2.
        MultipartHyphen2(multipart_hyphen2),

        /// Parsing multipart boundary.
        MultipartBoundary(multipart_boundary),

        /// Detecting multipart data parsing mechanism.
        MultipartDetectData(multipart_detect_data),

        /// Parsing multipart data by byte.
        MultipartDataByByte(multipart_data_by_byte),

        /// Parsing multipart data by content length.
        MultipartDataByLength(multipart_data_by_length),

        /// Parsing carriage return after data by length.
        MultipartDataByLengthCr(multipart_data_by_length_cr),

        /// Parsing line feed after data by length.
        MultipartDataByLengthLf(multipart_data_by_length_lf),

        /// Parsing potential line feed after data by byte.
        MultipartDataByByteLf(multipart_data_by_byte_lf),

        /// Parsing post boundary carriage return or hyphen.
        MultipartBoundaryCr(multipart_boundary_cr),

        /// Parsing post boundary line feed.
        MultipartBoundaryLf(multipart_boundary_lf),

        /// Parsing last boundary second hyphen that indicates end of multipart body.
        MultipartEnd(multipart_end),

        // ---------------------------------------------------------------------------------------------
        // URL ENCODED
        // ---------------------------------------------------------------------------------------------

        /// Parsing first byte of URL encoded name.
        FirstUrlEncodedName(first_url_encoded_name),

        /// Parsing URL encoded name.
        UrlEncodedName(url_encoded_name),

        /// Parsing URL encoded name hex sequence byte 1.
        UrlEncodedNameHex1(url_encoded_name_hex1),

        /// Parsing URL encoded name hex sequence byte 2.
        UrlEncodedNameHex2(url_encoded_name_hex2),

        /// Parsing URL encoded name plus sign.
        UrlEncodedNamePlus(url_encoded_name_plus),

        /// Parsing URL encoded value.
        UrlEncodedValue(url_encoded_value),

        /// Parsing URL encoded value hex sequence byte 1.
        UrlEncodedValueHex1(url_encoded_value_hex1),

        /// Parsing URL encoded value hex sequence byte 2.
        UrlEncodedValueHex2(url_encoded_value_hex2),

        /// Parsing URL encoded value plus sign.
        UrlEncodedValuePlus(url_encoded_value_plus),

        // ---------------------------------------------------------------------------------------------
        // FINISHED
        // ---------------------------------------------------------------------------------------------

        /// End of body parsing.
        BodyFinished(body_finished),

        /// Parsing entire message has finished.
        Finished(finished)
    }
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
