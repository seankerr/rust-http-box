# rust-http-box

![Build: Passing](https://img.shields.io/badge/build-passing-brightgreen.svg)
![dev: 0.1.2](https://img.shields.io/badge/dev-0.1.2-ff69b4.svg)
![license: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)

## What is http-box?

http-box is a push oriented HTTP 1 & 2 parser with the goal of remaining fast
and bare bones. Utilities are provided for handling additional HTTP details such
as query parsing, decoding hex encoded strings, and header field parsing.

The intent of http-box is to parse HTTP content and nothing more, so you will
not find `Request` or `Response` types within the library. They have not been
ruled out entirely, however, if they are provided at any future point, they will
be provided by a utility crate.

http-box will happily process any `&[u8]` data, and is not tied to any socket or
network dependencies.

## Features

- **New:** HTTP/2 support
- Push oriented and will process a single byte at a time
- Data is received via callback with the ability to break out of the parser loop
  at any point
- HTTP/1.x headers are normalized to lower-case
- Errors report type of error and on which byte it occurred
- Parse HTTP/1.x phases separately:
  - Head
    - Request / Response
    - Headers
  - Multipart
    - Headers
    - Data
  - Chunk Transfer-Encoded
    - Chunk Length
    - Extensions
    - Chunk Data
    - Trailers
  - URL encoded
    - Parameters
- Zero copy philosophy
- DoS protection is easily supported
- Fast!
- Use with any networking library

## API Documentation

https://docs.rs/http-box/0.1.3/http_box/

## HTTP/2.x Examples

## HTTP/1.x Examples

- [Intro](examples/http1_intro.md)
- [Parsing headers](examples/http1_head_parsing.md)

## HTTP Utilities

- [Utility functions](examples/http_utilities.md)
