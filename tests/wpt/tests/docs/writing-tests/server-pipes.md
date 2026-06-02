# wptserve Pipes

Pipes are designed to allow simple manipulation of the way that
static files are sent without requiring any custom code. They are also
useful for cross-origin tests because they can be used to activate a
substitution mechanism which can fill in details of ports and server
names in the setup on which the tests are being run.

## Enabling

Pipes are functions that may be used when serving files to alter parts
of the response. These are invoked by adding a pipe= query parameter
taking a | separated list of pipe functions and parameters. The pipe
functions are applied to the response from left to right. For example:

    GET /sample.txt?pipe=slice(1,200)|status(404).

This would serve bytes 1 to 199, inclusive, of foo.txt with the HTTP status
code 404.

Note: If you write directly to the response socket using ResponseWriter, or
when using the asis handler, only the trickle pipe will affect the response.

There are several built-in pipe functions, and it is possible to add
more using the `@pipe` decorator on a function, if required.

Note: Because of the way pipes compose, using some pipe functions prevents the
content-length of the response from being known in advance. In these cases the
server will close the connection to indicate the end of the response,
preventing the use of HTTP 1.1 keepalive.

## Built-In Pipes

### `sub`

Used to substitute variables from the server environment, or from the
request into the response. A typical use case is for testing
cross-domain since the exact domain name and ports of the servers are
generally unknown.

Substitutions are marked in a file using a block delimited by `{{`
and `}}`. Inside the block the following variables are available:

- `{{host}}` - The host name of the server excluding any subdomain part.
- `{{domains[]}}` - The domain name of a particular subdomain e.g.
  `{{domains[www]}}` for the `www` subdomain.
- `{{hosts[][]}}` - The domain name of a particular subdomain for a particular
  host. The first key may be empty (designating the "default" host) or the
  value `alt`; i.e., `{{hosts[alt][]}}` (designating the alternate host).
- `{{ports[][]}}` - The port number of servers, by protocol e.g.
  `{{ports[http][0]}}` for the first (and, depending on setup, possibly only)
  http server
- `{{headers[]}}` The HTTP headers in the request e.g. `{{headers[X-Test]}}`
  for a hypothetical `X-Test` header.
- `{{header_or_default(header, default)}}` The value of an HTTP header, or a
  default value if it is absent. e.g. `{{header_or_default(X-Test,
  test-header-absent)}}`
- `{{GET[]}}` The query parameters for the request e.g. `{{GET[id]}}` for an id
  parameter sent with the request.

So, for example, to write a JavaScript file called `xhr.js` that
depends on the host name of the server, without hardcoding, one might
write:

    var server_url = http://{{host}}:{{ports[http][0]}}/path/to/resource;
    //Create the actual XHR and so on

The file would then be included as:

    <script src="xhr.js?pipe=sub"></script>

This pipe can also be enabled by using a filename `*.sub.ext`, e.g. the file above could be called `xhr.sub.js`.

### `status`

Used to set the HTTP status of the response, for example:

    example.js?pipe=status(410)

### `headers`

Used to add or replace http headers in the response. Takes two or
three arguments; the header name, the header value and whether to
append the header rather than replace an existing header (default:
False). So, for example, a request for:

    example.html?pipe=header(Content-Type,text/plain)

causes example.html to be returned with a text/plain content type
whereas:

    example.html?pipe=header(Content-Type,text/plain,True)

Will cause example.html to be returned with both text/html and
text/plain content-type headers.

If the comma (`,`) or closing parenthesis (`)`) characters appear in the header
value, those characters  must be escaped with a backslash (`\`):

    example?pipe=header(Expires,Thu\,%2014%20Aug%201986%2018:00:00%20GMT)

(Note that the programming environment from which the request is issued may
require that the backslash character itself be escaped.)

### `slice`

Used to send only part of a response body. Takes the start and,
optionally, end bytes as arguments, although either can be null to
indicate the start or end of the file, respectively. So for example:

    example.txt?pipe=slice(10,20)

Would result in a response with a body containing 10 bytes of
example.txt including byte 10 but excluding byte 20.

    example.txt?pipe=slice(10)

Would cause all bytes from byte 10 of example.txt to be sent, but:

    example.txt?pipe=slice(null,20)

Would send the first 20 bytes of example.txt.

### `trickle`

Note: Using this function will force a connection close.

Used to send the body of a response in chunks with delays. Takes a
single argument that is a microsyntax consisting of colon-separated
commands. There are three types of commands:

* Bare numbers represent a number of bytes to send

* Numbers prefixed `d` indicate a delay in seconds

* Numbers prefixed `r` must only appear at the end of the command, and
  indicate that the preceding N items must be repeated until there is
  no more content to send. The number of items to repeat must be even.

In the absence of a repetition command, the entire remainder of the content is
sent at once when the command list is exhausted. So for example:

    example.txt?pipe=trickle(d1)

causes a 1s delay before sending the entirety of example.txt.

    example.txt?pipe=trickle(100:d1)

causes 100 bytes of example.txt to be sent, followed by a 1s delay,
and then the remainder of the file to be sent. On the other hand:

    example.txt?pipe=trickle(100:d1:r2)

Will cause the file to be sent in 100 byte chunks separated by a 1s
delay until the whole content has been sent.
