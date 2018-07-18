
Monkeypatching/mocking modules and environments
================================================================

.. currentmodule:: _pytest.monkeypatch

Sometimes tests need to invoke functionality which depends
on global settings or which invokes code which cannot be easily
tested such as network access.  The ``monkeypatch`` fixture
helps you to safely set/delete an attribute, dictionary item or
environment variable or to modify ``sys.path`` for importing.
See the `monkeypatch blog post`_ for some introduction material
and a discussion of its motivation.

.. _`monkeypatch blog post`: http://tetamap.wordpress.com/2009/03/03/monkeypatching-in-unit-tests-done-right/


Simple example: monkeypatching functions
---------------------------------------------------

If you want to pretend that ``os.expanduser`` returns a certain
directory, you can use the :py:meth:`monkeypatch.setattr` method to
patch this function before calling into a function which uses it::

    # content of test_module.py
    import os.path
    def getssh(): # pseudo application code
        return os.path.join(os.path.expanduser("~admin"), '.ssh')

    def test_mytest(monkeypatch):
        def mockreturn(path):
            return '/abc'
        monkeypatch.setattr(os.path, 'expanduser', mockreturn)
        x = getssh()
        assert x == '/abc/.ssh'

Here our test function monkeypatches ``os.path.expanduser`` and
then calls into a function that calls it.  After the test function
finishes the ``os.path.expanduser`` modification will be undone.

example: preventing "requests" from remote operations
------------------------------------------------------

If you want to prevent the "requests" library from performing http
requests in all your tests, you can do::

    # content of conftest.py
    import pytest
    @pytest.fixture(autouse=True)
    def no_requests(monkeypatch):
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

    See issue `#3290 <https://github.com/pytest-dev/pytest/issues/3290>`_ for details.


.. currentmodule:: _pytest.monkeypatch

API Reference
-------------

Consult the docs for the :class:`MonkeyPatch` class.
