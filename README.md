# rust-http-box

![Build: Passing](https://img.shields.io/badge/build-passing-brightgreen.svg)
![dev: 0.1.0](https://img.shields.io/badge/dev-0.1.0-ff69b4.svg)
![license: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)

rust-http-box is a fast push/callback oriented HTTP/1.1 (HTTP/2.0 coming soon) parser that works
only with slices of data, and never copies parsed data. Because of this, it is
possible to parse HTTP data one byte at a time. Parsing can be interrupted during any callback,
and at the end of each parsed slice.

This is purely an HTTP parsing library and is not tied to any networking framework. Use it to parse
stored HTTP request logs, test data, or to write a server and/or client.

Errors are handled intelligently letting you know what state the parser was in and which byte
triggered the error when it occurred.

## Progress

I'm in the stage of refactoring filenames, types, and overall locations of everything in the API.
Aside from this, the current changes are also in progress:

- MultipartHttp1Handler (the entire thing)

## Features

- Understands persistent requests
- Easily upgradable from HTTP/1.1 parsing to HTTP/2.0 in the same stream
- Header field and cookie name normalization to lower-case
- Parses:
  - Requests
  - Responses
  - Headers
  - Field values are turned into a HashSet: e.g. 'multipart/form-data; boundary="--ABCDEFG"'
  - Chunk encoded data
  - Query strings / URL encoded data
  - Multipart (in the works)

## Access To:

- Request:
  - Method
  - URL
  - Version
- Response:
  - Status
  - Status code
  - Version
- Headers (quoted and multi-line values are supported):
  - Cookies
  - Fields
  - Values
- Chunk encoded:
  - Size
  - Extension names
  - Extension values
  - Trailer fields
  - Trailer values
  - Raw data
- Multipart (in the works)
  - Header fields
  - Header values
  - File support
- URL encoded:
  - Fields
  - Values

## Performance

Currently rust-http-box is on par with the speeds seen from [fast-http](https://github.com/fukamachi/fast-http),
a Common Lisp HTTP parser, and significantly faster than the Joyant/NodeJS HTTP parser.

## API Documentation

http://metatomic.org/docs/api/http_box/index.html
