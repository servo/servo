# wdspec tests

The term "wdspec" describes a type of test in WPT which verifies some aspect of
[the WebDriver protocol](https://w3c.github.io/webdriver/). These tests are
written in [the Python programming language](https://www.python.org/) and
structured with [the pytest testing
framework](https://docs.pytest.org/en/latest/).

The test files are organized into subdirectories based on the WebDriver
command under test. For example, tests for [the Close Window
command](https://w3c.github.io/webdriver/#close-window) are located in then
`close_window` directory.

Similar to [testharness.js](testharness) tests, wdspec tests contain within
them any number of "sub-tests." Sub-tests are defined as Python functions whose
name begins with `test_`, e.g. `test_stale_element`.

## The `webdriver` client library

web-platform-tests maintains a WebDriver client library called `webdriver`
located in the `tools/webdriver/` directory. Like other client libraries, it
makes it easier to write code which interfaces with a browser using the
protocol.

Many tests require some "set up" code--logic intended to bring the browser to a
known state from which the expected behavior can be verified. The convenience
methods in the `webdriver` library **should** be used to perform this task
because they reduce duplication.

However, the same methods **should not** be used to issue the command under
test. Instead, the HTTP request describing the command should be sent directly.
This practice promotes the descriptive quality of the tests and limits
indirection that tends to obfuscate test failures.

Here is an example of a test for [the Element Click
command](https://w3c.github.io/webdriver/#element-click):

```python
from tests.support.asserts import assert_success

def test_null_response_value(session, inline):
    # The high-level API is used to set up a document and locate a click target
    session.url = inline("<p>foo")
    element = session.find.css("p", all=False)

    # An HTTP request is explicitly constructed for the "click" command itself
    response = session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/click".format(
            session_id=session.session_id,
            element_id=element.id))

    assert_success(response)
```

## Utility functions

The `wedbdriver` library is minimal by design. It mimics the structure of the
WebDriver specification. Many conformance tests perform similar operations
(e.g. calculating the center point of an element or creating a document), but
the library does not expose methods to facilitate them. Instead, wdspec tests
define shared functionality in the form of "support" files.

Many of these functions are intended to be used directly from the tests using
Python's built-in `import` keyword. Others (particularly those that operate on
a WebDriver session) are defined in terms of Pytest "fixtures" and must be
loaded accordingly. For more detail on how to define and use test fixtures,
please refer to [the pytest project's documentation on the
topic](https://docs.pytest.org/en/latest/fixture.html).
