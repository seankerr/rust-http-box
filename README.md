# rust-http-box

![Build: Passing](https://img.shields.io/badge/build-passing-brightgreen.svg)
![dev: 0.1.0](https://img.shields.io/badge/dev-0.1.0-ff69b4.svg)
![license: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)

## What is http-box?

http-box is a push oriented HTTP parser that processes a single byte at a time. It parses *&[u8]*
slices of data. It's not tied to a networking framework, which makes it suitable for more than
writing a web server or client library.

## Why a separate HTTP parser?

Because HTTP parsing support shouldn't be limited to libraries that are tied to networking layers.

Rustaceans already have multiple HTTP web servers, and multiple HTTP request/response libraries. The
concern with this is that they're all tied to different networking layers, or different parsers,
making it hard to move forward at the most rudimentary level.

Writing an HTTP parser should be a trivial, practically self-documenting task. And it should remain
separate from frameworks that use it.

## Features

- Push oriented and will process a single byte at a time.
- Callback oriented with the ability to halt parsing after each callback.
- Headers are normalized to lower-case.
- Option to parse phases separately from entire requests:
  - Status
  - Headers
  - Multipart / Chunked / URL encoded
- Zero copy philosophy.
- DoS protection is easily supported.
- Fast!

## Examples

- [Tracking State](examples/tracking_state.md)
- [Detecting Request or Response](examples/detect_request_response.md)
- [Headers](examples/headers.md)
- [Query Strings](examples/query.md)
- [Chunked Transfers](examples/chunked_transfer.md)

## API Documentation

http://metatomic.io/docs/api/http_box/index.html
