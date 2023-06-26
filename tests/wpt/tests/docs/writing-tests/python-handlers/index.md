# Python Handlers

Python file handlers are Python files which the server executes in response to
requests made to the corresponding URL. This is hooked up to a route like
`("*", "*.py", python_file_handler)`, meaning that any .py file will be
treated as a handler file (note that this makes it easy to write unsafe
handlers, particularly when running the server in a web-exposed setting).

The Python files must define a function named `main` with the signature:

    main(request, response)

...where `request` is [a wptserve `Request`
object](/tools/wptserve/docs/request) and `response` is [a wptserve `Response`
object](/tools/wptserve/docs/response).

This function must return a value in one of the following four formats:

    ((status_code, reason), headers, content)
    (status_code, headers, content)
    (headers, content)
    content

Above, `headers` is a list of (field name, value) pairs, and `content` is a
string or an iterable returning strings.

The `main` function may also update the response manually. For example, one may
use `response.headers.set` to set a response header, and only return the
content. One may even use this kind of handler, but manipulate the output
socket directly. The `writer` property of the response exposes a
`ResponseWriter` object that allows writing specific parts of the request or
direct access to the underlying socket. If used, the return value of the
`main` function and the properties of the `response` object will be ignored.

The wptserver implements a number of Python APIs for controlling traffic.

```eval_rst
.. toctree::
   :maxdepth: 1

   /tools/wptserve/docs/request
   /tools/wptserve/docs/response
   /tools/wptserve/docs/stash
```

### Importing local helper scripts

Python file handlers may import local helper scripts, e.g. to share logic
across multiple handlers. To avoid module name collision, however, imports must
be relative to the root of WPT. For example, in an imaginary
`cookies/resources/myhandler.py`:

```python
# DON'T DO THIS
import myhelper

# DO THIS
from cookies.resources import myhelper
```

Only absolute imports are allowed; do not use relative imports. If the path to
your helper script includes a hyphen (`-`), you can use `import_module` from
`importlib` to import it. For example:

```python
import importlib
myhelper = importlib.import_module('common.security-features.myhelper')
```

**Note on __init__ files**: Importing helper scripts like this
requires a 'path' of empty `__init__.py` files in every directory down
to the helper. For example, if your helper is
`css/css-align/resources/myhelper.py`, you need to have:

```
css/__init__.py
css/css-align/__init__.py
css/css-align/resources/__init__.py
```

## Example: Dynamic HTTP headers

The following code defines a Python handler that allows the requester to
control the value of the `Content-Type` HTTP response header:

```python
def main(request, response):
    content_type = request.GET.first('content-type')
    headers = [('Content-Type', content_type)]

    return (200, 'my status text'), headers, 'my response content'
```

If saved to a file named `resources/control-content-type.py`, the WPT server
will respond to requests for `resources/control-content-type.py` by executing
that code.

This could be used from a [testharness.js test](../testharness) like so:

```html
<!DOCTYPE html>
<meta charset="utf-8">
<title>Demonstrating the WPT server's Python handler feature</title>
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<script>
promise_test(function() {
  return fetch('resources/control-content-type.py?content-type=text/foobar')
    .then(function(response) {
      assert_equals(response.status, 200);
      assert_equals(response.statusText, 'my status text');
      assert_equals(response.headers.get('Content-Type'), 'text/foobar');
    });
});
</script>
```
