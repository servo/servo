# WebDriver client for Python

This package provides Python bindings
that conform to the [W3C WebDriver standard](https://w3c.github.io/webdriver/),
which specifies a remote control protocol for web browsers.

These bindings are written with determining
implementation compliance to the specification in mind,
so that different remote end drivers
can determine whether they meet the recognised standard.
The client is used for the WebDriver specification tests
in [web-platform-tests](https://github.com/web-platform-tests/wpt).

## Installation

To install the package individually
in your virtualenv or system-wide:

    % python setup.py install

Since this package does not have any external dependencies,
you can also use the client directly from the checkout directory,
which is useful if you want to contribute patches back:

    % cd /path/to/wdclient
    % python
    Python 2.7.12+ (default, Aug  4 2016, 20:04:34)
    [GCC 6.1.1 20160724] on linux2
    Type "help", "copyright", "credits" or "license" for more information.
    >>> import webdriver
    >>>

If you are writing WebDriver specification tests for
[WPT](https://github.com/web-platform-tests/wpt),
there is no need to install the client manually
as it is included in the `tools/webdriver` directory.

## Usage

You can use the built-in
[context manager](https://docs.python.org/2/reference/compound_stmts.html#the-with-statement)
to manage the lifetime of the session.
The session is started implicitly
at the first call to a command if it has not already been started,
and will implicitly be ended when exiting the context:

```py
import webdriver

with webdriver.Session("127.0.0.1", 4444) as session:
    session.url = "https://mozilla.org"
    print "The current URL is %s" % session.url
```

The following is functionally equivalent to the above,
but giving you manual control of the session:

```py
import webdriver

session = webdriver.Session("127.0.0.1", 4444)
session.start()

session.url = "https://mozilla.org"
print "The current URL is %s" % session.url

session.end()
```

## Dependencies

This client has the benefit of only using standard library dependencies.
No external PyPI dependencies are needed.
