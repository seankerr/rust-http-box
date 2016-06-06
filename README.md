# HTTP Box

HTTP Box is a zero-copy streaming HTTP parser, URL parser, and component decoder/encoder.

Making a fast, embeddable HTTP 1.1/2.x parser is the primary objective. HTTP Box is the perfect
library if you want to create a web server or HTTP proxy.

Development is currently in progress but moving forward steadily.

## Maintain Control

Each chunk of data is passed to a callback function. This callback function allows you to
prematurely exit the parser, but you remain in control, and can resume parsing at any point.
This will allow you to limit header and content size.

## Parsing Roadmap

- [x] Semi-Strict Parsing
- [x] Response Line
- [x] Request Line
- [x] Headers
  - [x] Multiline Header Values
  - [x] Quoted Header Values
- [x] Query String Parsing
- [x] URL Component Decoding
- [ ] Multipart Parsing
- [x] Chunked Decoding
