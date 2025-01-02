# wdspec tests

The term "wdspec" describes a type of test in WPT which verifies some aspect of
[WebDriver Classic](https://w3c.github.io/webdriver/) or
[WebDriver BiDi](https://w3c.github.io/webdriver-bidi) protocols. These tests are
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

## WebDriver BiDi

The wdspec tests for [WebDriver BiDi](https://w3c.github.io/webdriver-bidi) are
located in the `tests/bidi/` and `tests/interop` directories. Tests related to
external specifications are located in `external` subdirectories e.g.
[Permissions](https://www.w3.org/TR/permissions/) tests would go in
`tests/bidi/external/permissions/`.

The `webdriver.bidi.client.BidiSession` class provides an abstraction for the BiDi
client and contains properties corresponding to the
[WebDriver BiDi modules](https://w3c.github.io/webdriver-bidi/#protocol-modules). It
can be retrieved by fixture `bidi_session`.

### Extending WebDriver BiDi

This section describes how to extend the WebDriver BiDi client with an example of
adding support for [Permissions](https://www.w3.org/TR/permissions/).

#### Adding a New Module

##### Create `BidiModule`

BiDi modules are defined in the `tools/webdriver/webdriver/bidi/modules/` directory.
To add a new module called `permissions`, declare a Python class
`webdriver.bidi.modules.permissions.Permissions` that inherits from `BidiModule` and
store it in [`tools/webdriver/webdriver/bidi/modules/permissions.py`](https://github.com/web-platform-tests/wpt/blob/b81831169b8527a6c569a4ad92cf8a1baf4a7118/tools/webdriver/webdriver/bidi/modules/permissions.py#L7):

```python
class Permissions(BidiModule):
    pass
```

##### Import the Module in `bidi/modules/__init__.py`

Import this class in `tools/webdriver/webdriver/bidi/modules/__init__.py`:

```python
from .permissions import Permissions
```

##### Create an Instance of the Module in `webdriver.bidi.client.BidiSession`

Modify the `webdriver.bidi.client.BidiSession.__init__` method to create an instance
of `Permissions` and store it in a `permissions` [property](https://github.com/web-platform-tests/wpt/blob/b81831169b8527a6c569a4ad92cf8a1baf4a7118/tools/webdriver/webdriver/bidi/client.py#L98):

```python
self.permissions = modules.Permissions(self)
```

#### Adding a New Command

[WebDriver BiDi commands](https://w3c.github.io/webdriver-bidi/#commands) are
represented as module methods decorated with
`@command` (`webdriver.bidi.modules._module.command`). To add a new command, add
a method with the corresponding name (translated from camel case to snake case) to
the module. The method should return a dictionary that represents the
[command parameters](https://w3c.github.io/webdriver-bidi/#command-command-parameters).

For example, to add the
[`permissions.setPermission`](https://www.w3.org/TR/permissions/#webdriver-bidi-command-permissions-setPermission)
command, add the following `set_permission` method to the [`Permissions` class](https://github.com/web-platform-tests/wpt/blob/b81831169b8527a6c569a4ad92cf8a1baf4a7118/tools/webdriver/webdriver/bidi/modules/permissions.py#L9):

```python
from ._module import command
...
class Permissions(BidiModule):
...
    @command
    def set_permission(self,
          descriptor: Union[Optional[Mapping[str, Any]], Undefined] = UNDEFINED,
          state: Union[Optional[str], Undefined] = UNDEFINED,
          origin: Union[Optional[str], Undefined] = UNDEFINED,
          user_context: Union[Optional[str], Undefined] = UNDEFINED) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {
            "descriptor": descriptor,
            "state": state,
            "origin": origin,
            "userContext": user_context,
        }
        return params
```

### Adding Tests

Generally, a single test file should contain tests for a single parameter or feature
and stored in `webdriver/tests/bidi/{MODULE}/{METHOD}/{FEATURE}.py`
For example, tests for
[`permissions.setPermission`](https://www.w3.org/TR/permissions/#webdriver-bidi-command-permissions-setPermission)
could be split into:

* Invalid parameters: [`set_permission/invalid.py`](https://github.com/web-platform-tests/wpt/blob/aa019e3ff08cc75644edca41cfb095601477cb9d/webdriver/tests/bidi/external/permissions/set_permission/invalid.py)
* Common scenarios: [`set_permission/set_permission.py`](https://github.com/web-platform-tests/wpt/blob/aa019e3ff08cc75644edca41cfb095601477cb9d/webdriver/tests/bidi/external/permissions/set_permission/set_permission.py)
* User context: [`set_permission/user_context.py`](https://github.com/web-platform-tests/wpt/blob/master/webdriver/tests/bidi/external/permissions/set_permission/user_context.py)

Tests should use `bidi_session`'s modules' methods to send commands and verify its
side effects.
