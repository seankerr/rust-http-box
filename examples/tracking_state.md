# Tracking State

To keep [Parser](http://www.metatomic.io/docs/api/http_box/http1/struct.Parser.html) clean and easy
to maintain, it only has two jobs. The primary job is parsing data byte-by-byte. The second job is
executing callbacks. It is the callback implementor's job to track state.

Sometimes multiple states need to work together to produce a single result. A good example of this
is when headers are being parsed. The callback for the header name may be called multiple times in
order to receive the full header name. And the same is true for the header value. It isn't until the
header value is complete, that the header name/value pair can be stored.

This is where the [State](http://www.metatomic.io/docs/api/http_box/http1/enum.State.html) enum
comes into play. You can use this to track the current state when a callback is executed. There is
nothing mysterious about this enum. It's a helper type with the objective of simplifying state
tracking.
