# HTTP Box

HTTP Box is a zero-copy streaming HTTP parser, URL parser, and component decoder/encoder.

Making a fast, embeddable HTTP 1.1/2.x parser is the primary objective. HTTP Box is the perfect
library if you want to create a web server or HTTP proxy.

Development is currently in progress but moving forward steadily. Plan to have full HTTP 1.1
specification parsing finished by end of March 2016. Once complete, HTTP 2.x support will be
planned.

## Parsing Roadmap

- [x] Semi-Strict Parsing
- [x] Response Line
- [x] Request Line
- [x] Headers
  - [x] Multiline Header Values
  - [x] Quoted Header Values
- [x] URL Parsing
- [x] Query String Parsing
- [x] URL Component Decoding and Encoding
- [ ] Multipart Parsing
- [x] Chunked Decoding
