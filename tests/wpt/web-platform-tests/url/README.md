These tests are for browsers, but the data for
`a-element.html`, `url-constructor.html`, and `a-element-xhtml.xhtml`
is in `urltestdata.json` and can be re-used by non-browser implementations.
This file contains a JSON array of comments as strings and test cases as objects.
The keys for each test case are:

* `base`: an absolute URL as a string whose [parsing] without a base of its own should succeed.
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

[parsing]: https://url.spec.whatwg.org/#concept-basic-url-parser
[API]: https://url.spec.whatwg.org/#api

[`annevk/url`](https://github.com/annevk/url) hosts some other files that might be of
interest if you want to create additional tests.

Similar to `a-element.html` it would be trivial to add more tests for other objects that
expose links (e.g. URL and `<area>`). There's also room for enhancement and bits that
require independent tests:

* The encoding part of the URL parser
* The state override part of the URL parser (setting individual properties of a URL)
* Origin serialization
* `application/x-www-form-urlencoded`
