## HTTP Caching Tests

These tests cover HTTP-specified behaviours for caches, primarily from
[RFC7234](http://httpwg.org/specs/rfc7234.html), but as seen through the
lens of Fetch.

A few notes:

* By its nature, caching is optional; some tests expecting a response to be
  cached might fail because the client chose not to cache it, or chose to
  race the cache with a network request.

* Likewise, some tests might fail because there is a separate document-level
  cache that's ill-defined; see [this
  issue](https://github.com/whatwg/fetch/issues/354).

* [Partial content tests](partial.html) (a.k.a. Range requests) are not specified
  in Fetch; tests are included here for interest only.

* Some browser caches will behave differently when reloading /
  shift-reloading, despite the `cache mode` staying the same.

* At the moment, Edge doesn't appear to using HTTP caching in conjunction
  with Fetch at all.
