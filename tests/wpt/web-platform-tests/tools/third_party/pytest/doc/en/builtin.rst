
.. _`pytest helpers`:

Pytest API and builtin fixtures
================================================

This is a list of ``pytest.*`` API functions and fixtures.

For information on plugin hooks and objects, see :ref:`plugins`.

For information on the ``pytest.mark`` mechanism, see :ref:`mark`.

For the below objects, you can also interactively ask for help, e.g. by
typing on the Python interactive prompt something like::

    import pytest
    help(pytest)

.. currentmodule:: pytest

Invoking pytest interactively
---------------------------------------------------

.. autofunction:: main

More examples at :ref:`pytest.main-usage`


Helpers for assertions about Exceptions/Warnings
--------------------------------------------------------

.. autofunction:: raises

Examples at :ref:`assertraises`.

.. autofunction:: deprecated_call

Comparing floating point numbers
--------------------------------

.. autofunction:: approx

Raising a specific test outcome
--------------------------------------

You can use the following functions in your test, fixture or setup
functions to force a certain test outcome.  Note that most often
you can rather use declarative marks, see :ref:`skipping`.

.. autofunction:: _pytest.outcomes.fail
.. autofunction:: _pytest.outcomes.skip
.. autofunction:: _pytest.outcomes.importorskip
.. autofunction:: _pytest.outcomes.xfail
.. autofunction:: _pytest.outcomes.exit

Fixtures and requests
-----------------------------------------------------

To mark a fixture function:

.. autofunction:: _pytest.fixtures.fixture

Tutorial at :ref:`fixtures`.

The ``request`` object that can be used from fixture functions.

.. autoclass:: _pytest.fixtures.FixtureRequest()
    :members:


.. _builtinfixtures:
.. _builtinfuncargs:

Builtin fixtures/function arguments
-----------------------------------------

You can ask for available builtin or project-custom
:ref:`fixtures <fixtures>` by typing::

    $ pytest -q --fixtures
    cache
        Return a cache object that can persist state between testing sessions.
        
        cache.get(key, default)
        cache.set(key, value)
        
        Keys must be a ``/`` separated value, where the first part is usually the
        name of your plugin or application to avoid clashes with other cache users.
        
        Values can be any object handled by the json stdlib module.
    capsys
        Enable capturing of writes to sys.stdout/sys.stderr and make
        captured output available via ``capsys.readouterr()`` method calls
        which return a ``(out, err)`` tuple.  ``out`` and ``err`` will be ``text``
        objects.
    capsysbinary
        Enable capturing of writes to sys.stdout/sys.stderr and make
        captured output available via ``capsys.readouterr()`` method calls
        which return a ``(out, err)`` tuple.  ``out`` and ``err`` will be ``bytes``
        objects.
    capfd
        Enable capturing of writes to file descriptors 1 and 2 and make
        captured output available via ``capfd.readouterr()`` method calls
        which return a ``(out, err)`` tuple.  ``out`` and ``err`` will be ``text``
        objects.
    capfdbinary
        Enable capturing of write to file descriptors 1 and 2 and make
        captured output available via ``capfdbinary.readouterr`` method calls
        which return a ``(out, err)`` tuple.  ``out`` and ``err`` will be
        ``bytes`` objects.
    doctest_namespace
        Inject names into the doctest namespace.
    pytestconfig
        the pytest config object with access to command line opts.
    record_xml_property
        Add extra xml properties to the tag for the calling test.
        The fixture is callable with ``(name, value)``, with value being automatically
        xml-encoded.
    caplog
        Access and control log capturing.
        
        Captured logs are available through the following methods::
        
        * caplog.text()          -> string containing formatted log output
        * caplog.records()       -> list of logging.LogRecord instances
        * caplog.record_tuples() -> list of (logger_name, level, message) tuples
    monkeypatch
        The returned ``monkeypatch`` fixture provides these
        helper methods to modify objects, dictionaries or os.environ::
        
            monkeypatch.setattr(obj, name, value, raising=True)
            monkeypatch.delattr(obj, name, raising=True)
            monkeypatch.setitem(mapping, name, value)
            monkeypatch.delitem(obj, name, raising=True)
            monkeypatch.setenv(name, value, prepend=False)
            monkeypatch.delenv(name, value, raising=True)
            monkeypatch.syspath_prepend(path)
            monkeypatch.chdir(path)
        
        All modifications will be undone after the requesting
        test function or fixture has finished. The ``raising``
        parameter determines if a KeyError or AttributeError
        will be raised if the set/deletion operation has no target.
    recwarn
        Return a WarningsRecorder instance that provides these methods:
        
        * ``pop(category=None)``: return last warning matching the category.
        * ``clear()``: clear list of warnings
        
        See http://docs.python.org/library/warnings.html for information
        on warning categories.
    tmpdir_factory
        Return a TempdirFactory instance for the test session.
    tmpdir
        Return a temporary directory path object
        which is unique to each test function invocation,
        created as a sub directory of the base temporary
        directory.  The returned object is a `py.path.local`_
        path object.
    
    no tests ran in 0.12 seconds
