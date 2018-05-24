## urltestdata.json

These tests are for browsers, but the data for
`a-element.html`, `url-constructor.html`, `a-element-xhtml.xhtml`, and `failure.html`
is in `urltestdata.json` and can be re-used by non-browser implementations.
This file contains a JSON array of comments as strings and test cases as objects.
The keys for each test case are:

* `base`: an absolute URL as a string whose [parsing] without a base of its own must succeed.
  This key is always present,
  and may have a value like `"about:blank"` when `input` is an absolute URL.
* `input`: an URL as a string to be [parsed][parsing] with `base` as its base URL.
* Either:
  * `failure` with the value `true`, indicating that parsing `input` should return failure,
  * or `href`, `origin`, `protocol`, `username`, `password`, `host`, `hostname`, `port`,
    `pathname`, `search`, and `hash` with string values;
    indicating that parsing `input` should return an URL record
    and that the getters of each corresponding attribute in that URL’s [API]
    should return the corresponding value.

    The `origin` key may be missing.
    In that case, the API’s `origin` attribute is not tested.

In addition to testing that parsing `input` against `base` gives the result, a test harness for the
`URL` constructor (or similar APIs) should additionally test the following pattern: if `failure` is
true, parsing `about:blank` against `base` must give failure. This tests that the logic for
converting base URLs into strings properly fails the whole parsing algorithm if the base URL cannot
be parsed.

## URL parser's encoding argument

Tests in `/encoding` and `/html/infrastructure/urls/resolving-urls/query-encoding/` cover the
encoding argument to the URL parser.

[parsing]: https://url.spec.whatwg.org/#concept-basic-url-parser
[API]: https://url.spec.whatwg.org/#api
