# WebDriver client for Python

This package provides a WebDriver client compatible with
 the [W3C browser automation specification](https://w3c.github.io/webdriver/webdriver-spec.html).

The client is written with determining
implementation compliance to the specification in mind,
so that different remote end drivers
can determine whether they meet the recognised standard.
The client is used for the WebDriver specification tests
in the [Web Platform Tests](https://github.com/w3c/web-platform-tests).

## Installation

To install the package individually
in your virtualenv or system-wide::

    % python setup.py install

Or if you want to contribute patches::

    % python setup.py develop

If you are writing WebDriver specification tests for
[WPT](https://github.com/w3c/web-platform-tests),
there is no need to install the client manually
as it is picked up as a submodule to
[wpt-tools](https://github.com/w3c/wpt-tools)
that is checked out in `./tools`.

## Usage

```py
import webdriver

session = webdriver.Session("127.0.0.1", "4444")
session.start()

session.url = "https://mozilla.org"
print "The current URL is %s" % session.url

session.end()
```

## Dependencies

This client has the benefit of only using standard library dependencies.
No external PyPI dependencies are needed.
