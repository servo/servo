:orphan:

.. _`pytest helpers`:

Pytest API and builtin fixtures
================================================


Most of the information of this page has been moved over to :ref:`api-reference`.

For information on plugin hooks and objects, see :ref:`plugins`.

For information on the ``pytest.mark`` mechanism, see :ref:`mark`.

For information about fixtures, see :ref:`fixtures`. To see a complete list of available fixtures (add ``-v`` to also see fixtures with leading ``_``), type :

.. code-block:: pytest

    $ pytest  --fixtures -v
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-7.x.y, pluggy-1.x.y -- $PYTHON_PREFIX/bin/python
    cachedir: .pytest_cache
    rootdir: /home/sweet/project
    collected 0 items
    cache -- .../_pytest/cacheprovider.py:510
        Return a cache object that can persist state between testing sessions.

        cache.get(key, default)
        cache.set(key, value)

        Keys must be ``/`` separated strings, where the first part is usually the
        name of your plugin or application to avoid clashes with other cache users.

        Values can be any object handled by the json stdlib module.

    capsys -- .../_pytest/capture.py:878
        Enable text capturing of writes to ``sys.stdout`` and ``sys.stderr``.

        The captured output is made available via ``capsys.readouterr()`` method
        calls, which return a ``(out, err)`` namedtuple.
        ``out`` and ``err`` will be ``text`` objects.

    capsysbinary -- .../_pytest/capture.py:895
        Enable bytes capturing of writes to ``sys.stdout`` and ``sys.stderr``.

        The captured output is made available via ``capsysbinary.readouterr()``
        method calls, which return a ``(out, err)`` namedtuple.
        ``out`` and ``err`` will be ``bytes`` objects.

    capfd -- .../_pytest/capture.py:912
        Enable text capturing of writes to file descriptors ``1`` and ``2``.

        The captured output is made available via ``capfd.readouterr()`` method
        calls, which return a ``(out, err)`` namedtuple.
        ``out`` and ``err`` will be ``text`` objects.

    capfdbinary -- .../_pytest/capture.py:929
        Enable bytes capturing of writes to file descriptors ``1`` and ``2``.

        The captured output is made available via ``capfd.readouterr()`` method
        calls, which return a ``(out, err)`` namedtuple.
        ``out`` and ``err`` will be ``byte`` objects.

    doctest_namespace [session scope] -- .../_pytest/doctest.py:731
        Fixture that returns a :py:class:`dict` that will be injected into the
        namespace of doctests.

    pytestconfig [session scope] -- .../_pytest/fixtures.py:1365
        Session-scoped fixture that returns the session's :class:`pytest.Config`
        object.

        Example::

            def test_foo(pytestconfig):
                if pytestconfig.getoption("verbose") > 0:
                    ...

    record_property -- .../_pytest/junitxml.py:282
        Add extra properties to the calling test.

        User properties become part of the test report and are available to the
        configured reporters, like JUnit XML.

        The fixture is callable with ``name, value``. The value is automatically
        XML-encoded.

        Example::

            def test_function(record_property):
                record_property("example_key", 1)

    record_xml_attribute -- .../_pytest/junitxml.py:305
        Add extra xml attributes to the tag for the calling test.

        The fixture is callable with ``name, value``. The value is
        automatically XML-encoded.

    record_testsuite_property [session scope] -- .../_pytest/junitxml.py:343
        Record a new ``<property>`` tag as child of the root ``<testsuite>``.

        This is suitable to writing global information regarding the entire test
        suite, and is compatible with ``xunit2`` JUnit family.

        This is a ``session``-scoped fixture which is called with ``(name, value)``. Example:

        .. code-block:: python

            def test_foo(record_testsuite_property):
                record_testsuite_property("ARCH", "PPC")
                record_testsuite_property("STORAGE_TYPE", "CEPH")

        ``name`` must be a string, ``value`` will be converted to a string and properly xml-escaped.

        .. warning::

            Currently this fixture **does not work** with the
            `pytest-xdist <https://github.com/pytest-dev/pytest-xdist>`__ plugin. See
            :issue:`7767` for details.

    tmpdir_factory [session scope] -- .../_pytest/legacypath.py:295
        Return a :class:`pytest.TempdirFactory` instance for the test session.

    tmpdir -- .../_pytest/legacypath.py:302
        Return a temporary directory path object which is unique to each test
        function invocation, created as a sub directory of the base temporary
        directory.

        By default, a new base temporary directory is created each test session,
        and old bases are removed after 3 sessions, to aid in debugging. If
        ``--basetemp`` is used then it is cleared each session. See :ref:`base
        temporary directory`.

        The returned object is a `legacy_path`_ object.

        .. _legacy_path: https://py.readthedocs.io/en/latest/path.html

    caplog -- .../_pytest/logging.py:483
        Access and control log capturing.

        Captured logs are available through the following properties/methods::

        * caplog.messages        -> list of format-interpolated log messages
        * caplog.text            -> string containing formatted log output
        * caplog.records         -> list of logging.LogRecord instances
        * caplog.record_tuples   -> list of (logger_name, level, message) tuples
        * caplog.clear()         -> clear captured records and formatted log output string

    monkeypatch -- .../_pytest/monkeypatch.py:29
        A convenient fixture for monkey-patching.

        The fixture provides these methods to modify objects, dictionaries or
        os.environ::

            monkeypatch.setattr(obj, name, value, raising=True)
            monkeypatch.delattr(obj, name, raising=True)
            monkeypatch.setitem(mapping, name, value)
            monkeypatch.delitem(obj, name, raising=True)
            monkeypatch.setenv(name, value, prepend=None)
            monkeypatch.delenv(name, raising=True)
            monkeypatch.syspath_prepend(path)
            monkeypatch.chdir(path)

        All modifications will be undone after the requesting test function or
        fixture has finished. The ``raising`` parameter determines if a KeyError
        or AttributeError will be raised if the set/deletion operation has no target.

    recwarn -- .../_pytest/recwarn.py:29
        Return a :class:`WarningsRecorder` instance that records all warnings emitted by test functions.

        See https://docs.python.org/library/how-to/capture-warnings.html for information
        on warning categories.

    tmp_path_factory [session scope] -- .../_pytest/tmpdir.py:183
        Return a :class:`pytest.TempPathFactory` instance for the test session.

    tmp_path -- .../_pytest/tmpdir.py:198
        Return a temporary directory path object which is unique to each test
        function invocation, created as a sub directory of the base temporary
        directory.

        By default, a new base temporary directory is created each test session,
        and old bases are removed after 3 sessions, to aid in debugging. If
        ``--basetemp`` is used then it is cleared each session. See :ref:`base
        temporary directory`.

        The returned object is a :class:`pathlib.Path` object.


    ========================== no tests ran in 0.12s ===========================

You can also interactively ask for help, e.g. by typing on the Python interactive prompt something like:

.. code-block:: python

    import pytest

    help(pytest)
