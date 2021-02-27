# Server Features

For many tests, writing one or more static HTML files is
sufficient. However there are a large class of tests for which this
approach is insufficient, including:

* Tests that require cross-domain access

* Tests that depend on setting specific headers or status codes

* Tests that need to inspect the browser-sent request

* Tests that require state to be stored on the server

* Tests that require precise timing of the response.

To make writing such tests possible, we are using a number of
server-side components designed to make it easy to manipulate the
precise details of the response:

* *wptserve*, a custom Python HTTP server

* *pywebsocket*, an existing websockets server

* *tools/quic*, a custom Python 3 QUIC server using `aioquic`

wptserve is a Python-based web server. By default it serves static
files in the test suite. For more sophisticated requirements, several
mechanisms are available to take control of the response. These are
outlined below.

### Tests Involving Multiple Origins

Our test servers are guaranteed to be accessible through two domains
and five subdomains under each. The 'main' domain is unnamed; the
other is called 'alt'. These subdomains are: `www`, `www1`, `www2`,
`天気の良い日`, and `élève`; there is also `nonexistent` which is
guaranteed not to resolve. In addition, the HTTP server listens on two
ports, and the WebSockets server on one. These subdomains and ports
must be used for cross-origin tests.

Tests must not hardcode the hostname of the server that they expect to
be running on or the port numbers, as these are not guaranteed by the
test environment. Instead they can get this information in one of two
ways:

* From script, using the `location` API.

* By using a textual substitution feature of the server.

In order for the latter to work, a file must either have a name of the form
`{name}.sub.{ext}` e.g. `example-test.sub.html` or be referenced through a URL
containing `pipe=sub` in the query string e.g. `example-test.html?pipe=sub`.
The substitution syntax uses `{{ }}` to delimit items for substitution. For
example to substitute in the main host name, one would write: `{{host}}`.

To get full domains, including subdomains, there is the `hosts` dictionary,
where the first dimension is the name of the domain, and the second the
subdomain. For example, `{{hosts[][www]}}` would give the `www` subdomain under
the main (unnamed) domain, and `{{hosts[alt][élève]}}` would give the `élève`
subdomain under the alt domain.

For mostly historic reasons, the subdomains of the main domain are
also available under the `domains` dictionary; this is identical to
`hosts[]`.

Ports are also available on a per-protocol basis. For example,
`{{ports[ws][0]}}` is replaced with the first (and only) WebSockets port, while
`{{ports[http][1]}}` is replaced with the second HTTP port.

The request URL itself can be used as part of the substitution using the
`location` dictionary, which has entries matching the `window.location` API.
For example, `{{location[host]}}` is replaced by `hostname:port` for the
current request, matching `location.host`.


### Tests Requiring Special Headers

For tests requiring that a certain HTTP header is set to some static
value, a file with the same path as the test file except for an an
additional `.headers` suffix may be created. For example for
`/example/test.html`, the headers file would be
`/example/test.html.headers`. This file consists of lines of the form

    header-name: header-value

For example

    Content-Type: text/html; charset=big5

To apply the same headers to all files in a directory use a
`__dir__.headers` file. This will only apply to the immediate
directory and not subdirectories.

Headers files may be used in combination with substitutions by naming
the file e.g. `test.html.sub.headers`.


### Tests Requiring Full Control Over The HTTP Response

```eval_rst
.. toctree::
   :maxdepth: 1

   python-handlers/index
   server-pipes
```

For full control over the request and response, the server provides the ability
to write `.asis` files; these are served as literal HTTP responses. In other
words, they are sent byte-for-byte to the server without adding an HTTP status
line, headers, or anything else. This makes them suitable for testing
situations where the precise bytes on the wire are static, and control over the
timing is unnecessary, but the response does not conform to HTTP requirements.

The server also provides the ability to write [Python
"handlers"](python-handlers/index)--Python scripts that have access to request
data and can manipulate the content and timing of the response. Responses are
also influenced by [the `pipe` query string parameter](server-pipes).


### Tests Requiring HTTP/2.0

The server now has a prototype HTTP/2.0 server which gives you access to
some of the HTTP/2.0 specific functionality. Currently, the server is off
by default and needs to be run using `./wpt serve --h2` in order to enable it.
The HTTP/2.0 server supports handlers that work per-frame; these, along with the
API are documented in [Writing H2 Tests](h2tests).

> <b>Important:</b> The HTTP/2.0 server requires you to have
OpenSSL 1.0.2+. This is because HTTP/2.0 is negotiated using the
[TLS ALPN](https://tools.ietf.org/html/rfc7301) extension, which is only
supported in
[OpenSSL 1.0.2](https://www.openssl.org/news/openssl-1.0.2-notes.html) and up.


### Tests Requiring QUIC

We do not support loading a test over QUIC yet, but a test can establish a QUIC
connection to the test server (e.g. for WebTransport, similar to WebSocket).
Since the QUIC server is not yet enabled by default, tests must explicitly
declare that they need access to the QUIC server:

* For HTML tests (including testharness.js and reference tests), add the
  following element:
```html
<meta name="quic" content="true">
```
* For JavaScript tests (auto-generated tests), add the following comment:
```js
// META: quic=true
```

The QUIC server is not yet enabled by default, so QUIC tests will be skipped
unless `--enable-quic` is specified to `./wpt run`.

### Test Features specified as query params

Alternatively to specifying [Test Features](file-names.html#test-features) in
the test filename, they can be specified by setting the `wpt_flags` in the
[test variant](testharness.html#variants). For example, the following variant
will be loaded over HTTPS:
```html
<meta name="variant" content="?wpt_flags=https">
```

`https`, `h2` and `www` features are supported by `wpt_flags`.

Multiple features can be specified by having multiple `wpt_flags`. For example,
the following variant will be loaded over HTTPS and run on the www subdomain.

```html
<meta name="variant" content="wpt_flags=www&wpt_flags=https">
```
