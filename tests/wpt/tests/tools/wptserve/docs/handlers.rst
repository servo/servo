Handlers
========

Handlers are functions that have the general signature::

  handler(request, response)

It is expected that the handler will use information from
the request (e.g. the path) either to populate the response
object with the data to send, or to directly write to the
output stream via the ResponseWriter instance associated with
the request. If a handler writes to the output stream then the
server will not attempt additional writes, i.e. the choice to write
directly in the handler or not is all-or-nothing.

A number of general-purpose handler functions are provided by default:

.. _handlers.Python:

Python Handlers
---------------

Python handlers are functions which provide a higher-level API over
manually updating the response object, by causing the return value of
the function to provide (part of) the response. There are four
possible sets of values that may be returned::


  ((status_code, reason), headers, content)
  (status_code, headers, content)
  (headers, content)
  content

Here `status_code` is an integer status code, `headers` is a list of (field
name, value) pairs, and `content` is a string or an iterable returning strings.
Such a function may also update the response manually. For example one may use
`response.headers.set` to set a response header, and only return the content.
One may even use this kind of handler, but manipulate the output socket
directly, in which case the return value of the function, and the properties of
the response object, will be ignored.

The most common way to make a user function into a python handler is
to use the provided `wptserve.handlers.handler` decorator::

  from wptserve.handlers import handler

  @handler
  def test(request, response):
      return [("X-Test": "PASS"), ("Content-Type", "text/plain")], "test"

  #Later, assuming we have a Router object called 'router'

  router.register("GET", "/test", test)

JSON Handlers
-------------

This is a specialisation of the python handler type specifically
designed to facilitate providing JSON responses. The API is largely
the same as for a normal python handler, but the `content` part of the
return value is JSON encoded, and a default Content-Type header of
`application/json` is added. Again this handler is usually used as a
decorator::

  from wptserve.handlers import json_handler

  @json_handler
  def test(request, response):
      return {"test": "PASS"}

Python File Handlers
--------------------

Python file handlers are Python files which the server executes in response to
requests made to the corresponding URL. This is hooked up to a route like
``("*", "*.py", python_file_handler)``, meaning that any .py file will be
treated as a handler file (note that this makes it easy to write unsafe
handlers, particularly when running the server in a web-exposed setting).

The Python files must define a single function `main` with the signature::

  main(request, response)

This function then behaves just like those described in
:ref:`handlers.Python` above.

asis Handlers
-------------

These are used to serve files as literal byte streams including the
HTTP status line, headers and body. In the default configuration this
handler is invoked for all files with a .asis extension.

File Handlers
-------------

File handlers are used to serve static files. By default the content
type of these files is set by examining the file extension. However
this can be overridden, or additional headers supplied, by providing a
file with the same name as the file being served but an additional
.headers suffix, i.e. test.html has its headers set from
test.html.headers. The format of the .headers file is plaintext, with
each line containing::

  Header-Name: header_value

In addition headers can be set for a whole directory of files (but not
subdirectories), using a file called `__dir__.headers`.
