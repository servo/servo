Server
======

Basic server classes and router.

The following example creates a server that serves static files from
the `files` subdirectory of the current directory and causes it to
run on port 8080 until it is killed::

  from wptserve import server, handlers

  httpd = server.WebTestHttpd(port=8080, doc_root="./files/",
                              routes=[("GET", "*", handlers.file_handler)])
  httpd.start(block=True)

:mod:`Interface <wptserve>`
---------------------------

.. automodule:: wptserve.server
   :members:
