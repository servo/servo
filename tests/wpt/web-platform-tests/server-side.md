# Writing Complex Tests #

For many tests, writing one or more static HTML files is
sufficient. However there are a large class of tests for which this
approach is insufficient, including:

* Tests that require cross-domain access

* Tests that depend on setting specific headers or status codes

* Tests that need to inspect the browser sent request

* Tests that require state to be stored on the server

* Tests that require precise timing of the response.

To make writing such tests possible, we are using a number of
server-side components designed to make it easy to manipulate the
precise details of the response:

* *wptserve*, a custom python HTTP server.

* *pywebsocket*, an existing websockets server

This document will concentrate on the features of wptserve available
to test authors.

## Introduction to wptserve ##

wptserve is a python-based web server. By default it serves static
files in the testsuite. For more sophisticated requirements, several
mechanisms are available to take control of the response. These are
outlined below.

## Pipes ##

Suitable for:

 * Cross domain requests
 * Adding headers or status codes to static files
 * Controlling the sending of static file bodies

Pipes are designed to allow simple manipulation of the way that
static files are sent without requiring any custom code. They are also
useful for cross-origin tests because they can be used to activate a
substitution mechanism which can fill in details of ports and server
names in the setup on which the tests are being run.

Pipes are indicated by adding a query string to a request for a static
resource, with the parameter name `pipe`. The value of the query
should be a `|` serperated list of pipe functions. For example to
return a `.html` file with the status code 410 and a Content-Type of
text/plain, one might use:

    /resources/example.html?pipe=status(410)|header(Content-Type,text/plain)

There are a selection of pipe functions provided with wptserve and
more may be added if there are good use cases.

### sub ###

Used to subsitute variables from the server environment, or from the
request into the response. A typical use case is for testing
cross-domain since the exact domain name and ports of the servers are
generally unknown.

Substitutions are marked in a file using a block delimited by `{{`
and `}}`. Inside the block the following variables are avalible:

* `{{host}}` - the host name of the server exclusing any subdomain part.
* `{{domains[]}}` - the domain name of a particular subdomain
    e.g. `{{domains[www]}}` for the `www` subdomain.
* `{{ports[][]}}` - The port number of servers, by protocol
    e.g. `{{ports[http][1]}}` for the second (i.e. non-default) http
  server.
* `{{headers[]}}` - The HTTP headers in the request
    e.g. `{{headers[X-Test]}}` for a hypothetical `X-Test` header.
* `{{GET[]}}` - The query parameters for the request
    e.g. `{{GET[id]}}` for an id parameter sent with the request.

So, for example, to write a javascript file called `xhr.js` that does a
cross domain XHR test to a different subdomain and port, one would
write in the file:

    var server_url = "http://{{domains[www]}}:{{ports[http][1]}}/path/to/resource";
    //Create the actual XHR and so on

The file would then be included as:

    <script src="xhr.js?pipe=sub"></script>

### status ###

Used to set the HTTP status of the response, for example:

    example.js?pipe=status(410)

### headers ###

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

### slice ###

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

### trickle ###

Used to send the body of a response in chunks with delays. Takes a
single argument that is a microsyntax consisting of colon-separated
commands. There are three types of commands:

* Bare numbers represent a number of bytes to send

* Numbers prefixed `d` indicate a delay in seconds

* Numbers prefixed `r` must only appear at the end of the command, and
    indicate that the preceding N items must be repeated until there is
  no more content to send.

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

## asis files ##

Suitable for:

 * Static, HTTP-non-compliant responses

asis files are simply files with the extension `.asis`. They are sent
byte for byte to the server without adding a HTTP status line,
headers, or anything else. This makes them suitable for testing
situations where the precise bytes on the wire are static, and control
over the timing is unnecessary, but the response does not conform to
HTTP requirements.

## py files ##

Suitable for:

 * All tests requiring dynamic responses
 * Tests that need to store server side state.

The most flexible mechanism for writing tests is to use `.py`
files. These are interpreted as code and are suitable for the same
kinds of tasks that one might achieve using cgi, PHP or a similar
technology. Unlike cgi or PHP, the file is not executed directly and
does not produce output by writing to `stdout`. Instead files must
contain (at least) a function named `main`, with the signature:

    def main(request, response):
        pass

Here `request` is a `Request` object that contains details of the
request, and `response` is a `Response` object that can be used to set
properties of the response. Full details of these objects is
provided in the [wptserve documentation](http://wptserve.readthedocs.org/en/latest/).

In many cases tests will not need to work with the `response` object
directly. Instead they can set the status, headers and body simply by
returning values from the `main` function. If any value is returned,
it is interpreted as the response body. If two values are returned
they are interpreted as headers and body, and three values are
interpreted as status, headers, body. So, for example:

    def main(request, response):
        return "TEST"

creates a response with no non-default headers and the body
`TEST`. Headers can be added as follows:

    def main(request, response):
        return ([("Content-Type", "text/plain"), ("X-Test", "test")],
                "TEST")

And a status code as:

    def main(request, response):
        return (410,
                [("Content-Type", "text/plain"), ("X-Test", "test")],
              "TEST")

A custom status string may be returned by using a tuple `code, string`
in place of the code alone.

At the other end of the scale, some tests require precision over the
exact bytes sent over the wire and their timing. This can be achieved
using the `writer` property of the response, which exposes a
`ResponseWriter` object that allows wither writing specific parts of
the request or direct access to the underlying socket.

For full documentation on the facilities available in `.py` files, see
the [wptserve documentation](http://wptserve.readthedocs.org/en/latest/).
