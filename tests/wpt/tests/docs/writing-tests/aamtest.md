# aamtest tests

The aamtest tests are used to verify the mapping of web content to
browser-exposed platform-specific accessibility APIs. These mappings are
specified by working groups of the W3C in the following specifications:

* [Core-AAM](https://w3c.github.io/core-aam)
* [HTML-AAM](https://w3c.github.io/html-aam)
* [DPUB-AAM](https://w3c.github.io/dpub-aam)
* [MathML-AAM](https://w3c.github.io/mathml-aam)
* [Graphics-AAM](https://w3c.github.io/graphics-aam)
* [SVG-AAM](https://w3c.github.io/svg-aam/)

These tests are written in [the Python programming
language](https://www.python.org/) and structured with [the pytest testing
framework](https://docs.pytest.org/en/latest/).

The aamtest type is built on the [wdspec](wdspec) test type, and has access to
all the Python fixtures defined for wdspec tests. It uses the web-platform-tests
maintained WebDriver client library to load HTML and to send other WebDriver
commands to the browser.

The `wptrunner` will know a Python file is an aamtest if it is contained within
an `aamtests` directory.

## Platform-Specific Accessibility APIs

Accessibility APIs are platform (or operating system) specific, each platform
has their own API (sometimes more than one). Assistive technologies, such as
screen readers, interact with the browser on behalf of a user via these
APIs. The AAM specifications explain how to expose web content through these
APIs. You can read more about [the APIs in
Core-AAM](https://w3c.github.io/core-aam/#intro_aapi).

The table below lists:

* **API Name**: APIs supported by the aamtest framework
* **Fixture Name**: The name of the pytest fixture that returns access to the API (if you are
  on the correct platform).
* **Platform**: The platform of that API.
* **Python Bindings**: The Python library that provides bindings to query the API.

```eval_rst
.. list-table::
   :header-rows: 1

   * - API Name
     - Fixture Name
     - Platform
     - Python Bindings
   * - Accessibility Toolkit (`ATK <https://developer.gnome.org/atk/stable/>`_) and Assistive Technology Service Provider Interface (`AT-SPI <https://gnome.pages.gitlab.gnome.org/at-spi2-core/libatspi/>`_)
     - ``atspi``
     - Linux
     - `Provided through PyGObject <https://lazka.github.io/pgi-docs/#Atspi-2.0>`_
   * - The NSAccessibility Protocol for macOS (`AX API <https://developer.apple.com/documentation/appkit/nsaccessibility>`_)
     - ``axapi``
     - macOS
     - `pyobjc-framework-Accessibility <https://pypi.org/project/pyobjc-framework-Accessibility/>`_
   * - MSAA with IAccessible2 1.3 (`IA2 <https://wiki.linuxfoundation.org/accessibility/iaccessible2/start>`_)
     - ``ia2``
     - Windows
     - Loading module `ia2_api_all.idl <https://github.com/LinuxA11y/IAccessible2>`_ with `comtypes <https://pypi.org/project/comtypes/>`_
```

The APIs are exposed through a pytest fixture with the name in the table
above. The pytest fixture returns a wrapped version of the API. Requesting this
fixture on a platform where it is not supported will result in a `MISSING`
subtest. It is expected that each test file run on a given platform will have
one or more subtests that run to completion, as well as several subtests who's
results will not be recorded. This is because each test file should show how the
**same markup** is exposed in each supporting accessibility APIs/platforms.

### Package dependencies for Linux API AT-SPI Python Bindings

In order to test the Linux API AT-SPI, you need to have the following packages
installed:

```
sudo apt install libatspi2.0-dev libcairo2-dev libgirepository1.0-dev
```

## Adding new tests

If you would like to add a new aamtest to a specification that does not yet have
coverage, add the tests to an `aamtests` subfolder. This subfolder indicates the
Python files within it will be run as an aamtest. This includes restarting the
browser with accessibility enabled, if you are doing a full run of the test
suite.

In the `aamtests` subfolder, the `conftest.py` will need to add the
`webdriver/test/support` path and `core-aam/aamtests/support` path to the
sys.path and add the paths as `pytest_plugins` in order to have access to the
appropriate fixtures. See `core-aam/aamtests/conftest.py`.

### Test design

Similar to [testharness.js](testharness) tests, each file is a test, and a
function that begins with the name "test_" is a subtest of that test.

A typical test file contains some html markup and several subtests. Each subtest
will test how that markup is exposed in a single accessibility API. For example,
if you are testing how `<div role=foobar>foobar widget</div>` is exposed in
accessibility APIs, and foobars are supported in the Linux API AT-SPI and macOS
API AX API, you should add a subtest called `test_atspi` and `test_axapi`,
respectively. Both subtests will load the same HTML, then query their respective
accessibility APIs. The subtest name should include, at least, the name of the
API being tested for ease of understanding the test results.

For example, the file `foobar.py`:
```python
TEST_HTML = "<div role=foobar id=test></div>"

# Test of the Linux accessibility API: AT-SPI
def test_atspi(atspi, session, inline):
    # The `session` and `inline` fixtures are provided from the `wdspec` test infrastructure.
    session.url = inline(TEST_HTML)

    # The `atspi` fixture wraps the AT-SPI Python bindings and provides some helper functions,
    # such as `find_node`, which finds a node by DOM ID.
    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.FOOBAR

# Test of the macOS accessibility API: AX API
def test_axapi(axapi, session, inline):
    # The `session` and `inline` fixtures are provided from the `wdspec` test infrastructure.
    session.url = inline(TEST_HTML)

    # The `axapi` fixture wraps the AX API Python bindings and provides some helper functions,
    # such as `find_node`, which finds a node by DOM ID.
    node = axapi.find_node("test", session.url)
    role = axapi.AXUIElementCopyAttributeValue(node, "AXRole", None)[1]
    assert role == "AXFoobar"
```

## Adding support for an unsupported API

To add an unsupported API:

1. Add the Python package that provides the Python bindings for that API to
   `tools/wptrunner/requirements_platform_accessibility.txt`.
2. Create a wrapper object for the new API in `newapi_wrapper.py` in
   `core-aam/aamtests/support/`. It must inherit from `ApiWrapper` and follow
   the same conventions as the other API wrappers, as appropriate.
3. Add the fixture for that API in
   `core-aam/aamtests/support/fixtures_a11y_api.py`.
4. Update the table in the "Platform-Specific Accessibility APIs" section of this
   document.
5. Add a new subtest to all the files that contain markup you would like to test.
