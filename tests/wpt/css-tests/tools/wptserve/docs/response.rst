Response
========

Response object. This object is used to control the response that will
be sent to the HTTP client. A handler function will take the response
object and fill in various parts of the response. For example, a plain
text response with the body 'Some example content' could be produced as::

  def handler(request, response):
      response.headers.set("Content-Type", "text/plain")
      response.content = "Some example content"

The response object also gives access to a ResponseWriter, which
allows direct access to the response socket. For example, one could
write a similar response but with more explicit control as follows::

  import time

  def handler(request, response):
      response.add_required_headers = False # Don't implicitly add HTTP headers
      response.writer.write_status(200)
      response.writer.write_header("Content-Type", "text/plain")
      response.writer.write_header("Content-Length", len("Some example content"))
      response.writer.end_headers()
      response.writer.write("Some ")
      time.sleep(1)
      response.writer.write("example content")

Note that when writing the response directly like this it is always
necessary to either set the Content-Length header or set
`response.close_connection = True`. Without one of these, the client
will not be able to determine where the response body ends and will
continue to load indefinitely.

.. _response.Interface:

:mod:`Interface <response>`
---------------------------

.. automodule:: wptserve.response
   :members:
