.. _monkeypatching:

How to monkeypatch/mock modules and environments
================================================================

.. currentmodule:: pytest

Sometimes tests need to invoke functionality which depends
on global settings or which invokes code which cannot be easily
tested such as network access.  The ``monkeypatch`` fixture
helps you to safely set/delete an attribute, dictionary item or
environment variable, or to modify ``sys.path`` for importing.

The ``monkeypatch`` fixture provides these helper methods for safely patching and mocking
functionality in tests:

* :meth:`monkeypatch.setattr(obj, name, value, raising=True) <pytest.MonkeyPatch.setattr>`
* :meth:`monkeypatch.delattr(obj, name, raising=True) <pytest.MonkeyPatch.delattr>`
* :meth:`monkeypatch.setitem(mapping, name, value) <pytest.MonkeyPatch.setitem>`
* :meth:`monkeypatch.delitem(obj, name, raising=True) <pytest.MonkeyPatch.delitem>`
* :meth:`monkeypatch.setenv(name, value, prepend=None) <pytest.MonkeyPatch.setenv>`
* :meth:`monkeypatch.delenv(name, raising=True) <pytest.MonkeyPatch.delenv>`
* :meth:`monkeypatch.syspath_prepend(path) <pytest.MonkeyPatch.syspath_prepend>`
* :meth:`monkeypatch.chdir(path) <pytest.MonkeyPatch.chdir>`
* :meth:`monkeypatch.context() <pytest.MonkeyPatch.context>`


All modifications will be undone after the requesting
test function or fixture has finished. The ``raising``
parameter determines if a ``KeyError`` or ``AttributeError``
will be raised if the target of the set/deletion operation does not exist.

Consider the following scenarios:

1. Modifying the behavior of a function or the property of a class for a test e.g.
there is an API call or database connection you will not make for a test but you know
what the expected output should be. Use :py:meth:`monkeypatch.setattr <MonkeyPatch.setattr>` to patch the
function or property with your desired testing behavior. This can include your own functions.
Use :py:meth:`monkeypatch.delattr <MonkeyPatch.delattr>` to remove the function or property for the test.

2. Modifying the values of dictionaries e.g. you have a global configuration that
you want to modify for certain test cases. Use :py:meth:`monkeypatch.setitem <MonkeyPatch.setitem>` to patch the
dictionary for the test. :py:meth:`monkeypatch.delitem <MonkeyPatch.delitem>` can be used to remove items.

3. Modifying environment variables for a test e.g. to test program behavior if an
environment variable is missing, or to set multiple values to a known variable.
:py:meth:`monkeypatch.setenv <MonkeyPatch.setenv>` and :py:meth:`monkeypatch.delenv <MonkeyPatch.delenv>` can be used for
these patches.

4. Use ``monkeypatch.setenv("PATH", value, prepend=os.pathsep)`` to modify ``$PATH``, and
:py:meth:`monkeypatch.chdir <MonkeyPatch.chdir>` to change the context of the current working directory
during a test.

5. Use :py:meth:`monkeypatch.syspath_prepend <MonkeyPatch.syspath_prepend>` to modify ``sys.path`` which will also
call ``pkg_resources.fixup_namespace_packages`` and :py:func:`importlib.invalidate_caches`.

6. Use :py:meth:`monkeypatch.context <MonkeyPatch.context>` to apply patches only in a specific scope, which can help
control teardown of complex fixtures or patches to the stdlib.

See the `monkeypatch blog post`_ for some introduction material
and a discussion of its motivation.

.. _`monkeypatch blog post`: https://tetamap.wordpress.com//2009/03/03/monkeypatching-in-unit-tests-done-right/

Monkeypatching functions
------------------------

Consider a scenario where you are working with user directories. In the context of
testing, you do not want your test to depend on the running user. ``monkeypatch``
can be used to patch functions dependent on the user to always return a
specific value.

In this example, :py:meth:`monkeypatch.setattr <MonkeyPatch.setattr>` is used to patch ``Path.home``
so that the known testing path ``Path("/abc")`` is always used when the test is run.
This removes any dependency on the running user for testing purposes.
:py:meth:`monkeypatch.setattr <MonkeyPatch.setattr>` must be called before the function which will use
the patched function is called.
After the test function finishes the ``Path.home`` modification will be undone.

.. code-block:: python

    # contents of test_module.py with source code and the test
    from pathlib import Path


    def getssh():
        """Simple function to return expanded homedir ssh path."""
        return Path.home() / ".ssh"


    def test_getssh(monkeypatch):
        # mocked return function to replace Path.home
        # always return '/abc'
        def mockreturn():
            return Path("/abc")

        # Application of the monkeypatch to replace Path.home
        # with the behavior of mockreturn defined above.
        monkeypatch.setattr(Path, "home", mockreturn)

        # Calling getssh() will use mockreturn in place of Path.home
        # for this test with the monkeypatch.
        x = getssh()
        assert x == Path("/abc/.ssh")

Monkeypatching returned objects: building mock classes
------------------------------------------------------

:py:meth:`monkeypatch.setattr <MonkeyPatch.setattr>` can be used in conjunction with classes to mock returned
objects from functions instead of values.
Imagine a simple function to take an API url and return the json response.

.. code-block:: python

    # contents of app.py, a simple API retrieval example
    import requests


    def get_json(url):
        """Takes a URL, and returns the JSON."""
        r = requests.get(url)
        return r.json()

We need to mock ``r``, the returned response object for testing purposes.
The mock of ``r`` needs a ``.json()`` method which returns a dictionary.
This can be done in our test file by defining a class to represent ``r``.

.. code-block:: python

    # contents of test_app.py, a simple test for our API retrieval
    # import requests for the purposes of monkeypatching
    import requests

    # our app.py that includes the get_json() function
    # this is the previous code block example
    import app


    # custom class to be the mock return value
    # will override the requests.Response returned from requests.get
    class MockResponse:
        # mock json() method always returns a specific testing dictionary
        @staticmethod
        def json():
            return {"mock_key": "mock_response"}


    def test_get_json(monkeypatch):
        # Any arguments may be passed and mock_get() will always return our
        # mocked object, which only has the .json() method.
        def mock_get(*args, **kwargs):
            return MockResponse()

        # apply the monkeypatch for requests.get to mock_get
        monkeypatch.setattr(requests, "get", mock_get)

        # app.get_json, which contains requests.get, uses the monkeypatch
        result = app.get_json("https://fakeurl")
        assert result["mock_key"] == "mock_response"


``monkeypatch`` applies the mock for ``requests.get`` with our ``mock_get`` function.
The ``mock_get`` function returns an instance of the ``MockResponse`` class, which
has a ``json()`` method defined to return a known testing dictionary and does not
require any outside API connection.

You can build the ``MockResponse`` class with the appropriate degree of complexity for
the scenario you are testing. For instance, it could include an ``ok`` property that
always returns ``True``, or return different values from the ``json()`` mocked method
based on input strings.

This mock can be shared across tests using a ``fixture``:

.. code-block:: python

    # contents of test_app.py, a simple test for our API retrieval
    import pytest
    import requests

    # app.py that includes the get_json() function
    import app


    # custom class to be the mock return value of requests.get()
    class MockResponse:
        @staticmethod
        def json():
            return {"mock_key": "mock_response"}


    # monkeypatched requests.get moved to a fixture
    @pytest.fixture
    def mock_response(monkeypatch):
        """Requests.get() mocked to return {'mock_key':'mock_response'}."""

        def mock_get(*args, **kwargs):
            return MockResponse()

        monkeypatch.setattr(requests, "get", mock_get)


    # notice our test uses the custom fixture instead of monkeypatch directly
    def test_get_json(mock_response):
        result = app.get_json("https://fakeurl")
        assert result["mock_key"] == "mock_response"


Furthermore, if the mock was designed to be applied to all tests, the ``fixture`` could
be moved to a ``conftest.py`` file and use the with ``autouse=True`` option.


Global patch example: preventing "requests" from remote operations
------------------------------------------------------------------

If you want to prevent the "requests" library from performing http
requests in all your tests, you can do:

.. code-block:: python

    # contents of conftest.py
    import pytest


    @pytest.fixture(autouse=True)
    def no_requests(monkeypatch):
        """Remove requests.sessions.Session.request for all tests."""
        monkeypatch.delattr("requests.sessions.Session.request")

This autouse fixture will be executed for each test function and it
will delete the method ``request.session.Session.request``
so that any attempts within tests to create http requests will fail.


.. note::

    Be advised that it is not recommended to patch builtin functions such as ``open``,
    ``compile``, etc., because it might break pytest's internals. If that's
    unavoidable, passing ``--tb=native``, ``--assert=plain`` and ``--capture=no`` might
    help although there's no guarantee.

.. note::

    Mind that patching ``stdlib`` functions and some third-party libraries used by pytest
    might break pytest itself, therefore in those cases it is recommended to use
    :meth:`MonkeyPatch.context` to limit the patching to the block you want tested:

    .. code-block:: python

        import functools


        def test_partial(monkeypatch):
            with monkeypatch.context() as m:
                m.setattr(functools, "partial", 3)
                assert functools.partial == 3

    See :issue:`3290` for details.


Monkeypatching environment variables
------------------------------------

If you are working with environment variables you often need to safely change the values
or delete them from the system for testing purposes. ``monkeypatch`` provides a mechanism
to do this using the ``setenv`` and ``delenv`` method. Our example code to test:

.. code-block:: python

    # contents of our original code file e.g. code.py
    import os


    def get_os_user_lower():
        """Simple retrieval function.
        Returns lowercase USER or raises OSError."""
        username = os.getenv("USER")

        if username is None:
            raise OSError("USER environment is not set.")

        return username.lower()

There are two potential paths. First, the ``USER`` environment variable is set to a
value. Second, the ``USER`` environment variable does not exist. Using ``monkeypatch``
both paths can be safely tested without impacting the running environment:

.. code-block:: python

    # contents of our test file e.g. test_code.py
    import pytest


    def test_upper_to_lower(monkeypatch):
        """Set the USER env var to assert the behavior."""
        monkeypatch.setenv("USER", "TestingUser")
        assert get_os_user_lower() == "testinguser"


    def test_raise_exception(monkeypatch):
        """Remove the USER env var and assert OSError is raised."""
        monkeypatch.delenv("USER", raising=False)

        with pytest.raises(OSError):
            _ = get_os_user_lower()

This behavior can be moved into ``fixture`` structures and shared across tests:

.. code-block:: python

    # contents of our test file e.g. test_code.py
    import pytest


    @pytest.fixture
    def mock_env_user(monkeypatch):
        monkeypatch.setenv("USER", "TestingUser")


    @pytest.fixture
    def mock_env_missing(monkeypatch):
        monkeypatch.delenv("USER", raising=False)


    # notice the tests reference the fixtures for mocks
    def test_upper_to_lower(mock_env_user):
        assert get_os_user_lower() == "testinguser"


    def test_raise_exception(mock_env_missing):
        with pytest.raises(OSError):
            _ = get_os_user_lower()


Monkeypatching dictionaries
---------------------------

:py:meth:`monkeypatch.setitem <MonkeyPatch.setitem>` can be used to safely set the values of dictionaries
to specific values during tests. Take this simplified connection string example:

.. code-block:: python

    # contents of app.py to generate a simple connection string
    DEFAULT_CONFIG = {"user": "user1", "database": "db1"}


    def create_connection_string(config=None):
        """Creates a connection string from input or defaults."""
        config = config or DEFAULT_CONFIG
        return f"User Id={config['user']}; Location={config['database']};"

For testing purposes we can patch the ``DEFAULT_CONFIG`` dictionary to specific values.

.. code-block:: python

    # contents of test_app.py
    # app.py with the connection string function (prior code block)
    import app


    def test_connection(monkeypatch):
        # Patch the values of DEFAULT_CONFIG to specific
        # testing values only for this test.
        monkeypatch.setitem(app.DEFAULT_CONFIG, "user", "test_user")
        monkeypatch.setitem(app.DEFAULT_CONFIG, "database", "test_db")

        # expected result based on the mocks
        expected = "User Id=test_user; Location=test_db;"

        # the test uses the monkeypatched dictionary settings
        result = app.create_connection_string()
        assert result == expected

You can use the :py:meth:`monkeypatch.delitem <MonkeyPatch.delitem>` to remove values.

.. code-block:: python

    # contents of test_app.py
    import pytest

    # app.py with the connection string function
    import app


    def test_missing_user(monkeypatch):
        # patch the DEFAULT_CONFIG t be missing the 'user' key
        monkeypatch.delitem(app.DEFAULT_CONFIG, "user", raising=False)

        # Key error expected because a config is not passed, and the
        # default is now missing the 'user' entry.
        with pytest.raises(KeyError):
            _ = app.create_connection_string()


The modularity of fixtures gives you the flexibility to define
separate fixtures for each potential mock and reference them in the needed tests.

.. code-block:: python

    # contents of test_app.py
    import pytest

    # app.py with the connection string function
    import app


    # all of the mocks are moved into separated fixtures
    @pytest.fixture
    def mock_test_user(monkeypatch):
        """Set the DEFAULT_CONFIG user to test_user."""
        monkeypatch.setitem(app.DEFAULT_CONFIG, "user", "test_user")


    @pytest.fixture
    def mock_test_database(monkeypatch):
        """Set the DEFAULT_CONFIG database to test_db."""
        monkeypatch.setitem(app.DEFAULT_CONFIG, "database", "test_db")


    @pytest.fixture
    def mock_missing_default_user(monkeypatch):
        """Remove the user key from DEFAULT_CONFIG"""
        monkeypatch.delitem(app.DEFAULT_CONFIG, "user", raising=False)


    # tests reference only the fixture mocks that are needed
    def test_connection(mock_test_user, mock_test_database):
        expected = "User Id=test_user; Location=test_db;"

        result = app.create_connection_string()
        assert result == expected


    def test_missing_user(mock_missing_default_user):
        with pytest.raises(KeyError):
            _ = app.create_connection_string()


.. currentmodule:: pytest

API Reference
-------------

Consult the docs for the :class:`MonkeyPatch` class.
