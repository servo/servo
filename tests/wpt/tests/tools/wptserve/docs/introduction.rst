Introduction
============

wptserve has been designed with the specific goal of making a server
that is suitable for writing tests for the web platform. This means
that it cannot use common abstractions over HTTP such as WSGI, since
these assume that the goal is to generate a well-formed HTTP
response. Testcases, however, often require precise control of the
exact bytes sent over the wire and their timing. The full list of
design goals for the server are:

* Suitable to run on individual test machines and over the public internet.

* Support plain TCP and SSL servers.

* Serve static files with the minimum of configuration.

* Allow headers to be overwritten on a per-file and per-directory
  basis.

* Full customisation of headers sent (e.g. altering or omitting
  "mandatory" headers).

* Simple per-client state.

* Complex logic in tests, up to precise control over the individual
  bytes sent and the timing of sending them.

Request Handling
----------------

At the high level, the design of the server is based around similar
concepts to those found in common web frameworks like Django, Pyramid
or Flask. In particular the lifecycle of a typical request will be
familiar to users of these systems. Incoming requests are parsed and a
:doc:`Request <request>` object is constructed. This object is passed
to a :ref:`Router <router.Interface>` instance, which is
responsible for mapping the request method and path to a handler
function. This handler is passed two arguments; the request object and
a :doc:`Response <response>` object. In cases where only simple
responses are required, the handler function may fill in the
properties of the response object and the server will take care of
constructing the response. However each Response also contains a
:ref:`ResponseWriter <response.Interface>` which can be
used to directly control the TCP socket.

By default there are several built-in handler functions that provide a
higher level API than direct manipulation of the Response
object. These are documented in :doc:`handlers`.


