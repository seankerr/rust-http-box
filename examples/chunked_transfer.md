# Chunked Transfers

[HttpHandler](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html) has 8 callback
functions related to chunked transfer encoding:

- [on_body_finished()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_body_finished): Indicates all chunk data has been parsed
- [on_chunk_data()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_chunk_data): Receive chunk data
- [on_chunk_extension_finished()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_chunk_extension_finished): Indicates the current chunk extension has finished parsing
- [on_chunk_extension_name()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_chunk_extension_name): Receive chunk extension name
- [on_chunk_extension_value()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_chunk_extension_value): Receive chunk extension value
- [on_chunk_length()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_chunk_length): Receive current chunk length
- [on_header_name()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_header_name): Receive trailer name details
- [on_header_value()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_header_value): Receive trailer value details
- [on_headers_finished()](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html#method.on_headers_finished): Indicates trailers have finished parsing
