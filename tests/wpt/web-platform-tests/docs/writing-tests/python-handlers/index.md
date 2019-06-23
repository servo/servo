# Python Handlers

Python file handlers are Python files which the server executes in response to
requests made to the corresponding URL. This is hooked up to a route like
`("*", "*.py", python_file_handler)`, meaning that any .py file will be
treated as a handler file (note that this makes it easy to write unsafe
handlers, particularly when running the server in a web-exposed setting).

The Python files must define a single function `main` with the signature::

    main(request, response)

The wptserver implements a number of Python APIs for controlling traffic.

```eval_rst
.. toctree::
   :maxdepth: 1

   /tools/wptserve/docs/request
   /tools/wptserve/docs/response
   /tools/wptserve/docs/stash
```
