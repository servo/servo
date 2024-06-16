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
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y -- $PYTHON_PREFIX/bin/python
    cachedir: .pytest_cache
    rootdir: /home/sweet/project
    collected 0 items
    cache -- .../_pytest/cacheprovider.py:549
        Return a cache object that can persist state between testing sessions.

        cache.get(key, default)
        cache.set(key, value)

        Keys must be ``/`` separated strings, where the first part is usually the
        name of your plugin or application to avoid clashes with other cache users.

        Values can be any object handled by the json stdlib module.

    capsysbinary -- .../_pytest/capture.py:1003
        Enable bytes capturing of writes to ``sys.stdout`` and ``sys.stderr``.

        The captured output is made available via ``capsysbinary.readouterr()``
        method calls, which return a ``(out, err)`` namedtuple.
        ``out`` and ``err`` will be ``bytes`` objects.

        Returns an instance of :class:`CaptureFixture[bytes] <pytest.CaptureFixture>`.

        Example:
        .. code-block:: python

            def test_output(capsysbinary):
                print("hello")
                captured = capsysbinary.readouterr()
                assert captured.out == b"hello\n"

    capfd -- .../_pytest/capture.py:1030
        Enable text capturing of writes to file descriptors ``1`` and ``2``.

        The captured output is made available via ``capfd.readouterr()`` method
        calls, which return a ``(out, err)`` namedtuple.
        ``out`` and ``err`` will be ``text`` objects.

        Returns an instance of :class:`CaptureFixture[str] <pytest.CaptureFixture>`.

        Example:
        .. code-block:: python

            def test_system_echo(capfd):
                os.system('echo "hello"')
                captured = capfd.readouterr()
                assert captured.out == "hello\n"

    capfdbinary -- .../_pytest/capture.py:1057
        Enable bytes capturing of writes to file descriptors ``1`` and ``2``.

        The captured output is made available via ``capfd.readouterr()`` method
        calls, which return a ``(out, err)`` namedtuple.
        ``out`` and ``err`` will be ``byte`` objects.

        Returns an instance of :class:`CaptureFixture[bytes] <pytest.CaptureFixture>`.

        Example:
        .. code-block:: python

            def test_system_echo(capfdbinary):
                os.system('echo "hello"')
                captured = capfdbinary.readouterr()
                assert captured.out == b"hello\n"

    capsys -- .../_pytest/capture.py:976
        Enable text capturing of writes to ``sys.stdout`` and ``sys.stderr``.

        The captured output is made available via ``capsys.readouterr()`` method
        calls, which return a ``(out, err)`` namedtuple.
        ``out`` and ``err`` will be ``text`` objects.

        Returns an instance of :class:`CaptureFixture[str] <pytest.CaptureFixture>`.

        Example:
        .. code-block:: python

            def test_output(capsys):
                print("hello")
                captured = capsys.readouterr()
                assert captured.out == "hello\n"

    doctest_namespace [session scope] -- .../_pytest/doctest.py:738
        Fixture that returns a :py:class:`dict` that will be injected into the
        namespace of doctests.

        Usually this fixture is used in conjunction with another ``autouse`` fixture:

        .. code-block:: python

            @pytest.fixture(autouse=True)
            def add_np(doctest_namespace):
                doctest_namespace["np"] = numpy

        For more details: :ref:`doctest_namespace`.

    pytestconfig [session scope] -- .../_pytest/fixtures.py:1335
        Session-scoped fixture that returns the session's :class:`pytest.Config`
        object.

        Example::

            def test_foo(pytestconfig):
                if pytestconfig.getoption("verbose") > 0:
                    ...

    record_property -- .../_pytest/junitxml.py:284
        Add extra properties to the calling test.

        User properties become part of the test report and are available to the
        configured reporters, like JUnit XML.

        The fixture is callable with ``name, value``. The value is automatically
        XML-encoded.

        Example::

            def test_function(record_property):
                record_property("example_key", 1)

    record_xml_attribute -- .../_pytest/junitxml.py:307
        Add extra xml attributes to the tag for the calling test.

        The fixture is callable with ``name, value``. The value is
        automatically XML-encoded.

    record_testsuite_property [session scope] -- .../_pytest/junitxml.py:345
        Record a new ``<property>`` tag as child of the root ``<testsuite>``.

        This is suitable to writing global information regarding the entire test
        suite, and is compatible with ``xunit2`` JUnit family.

        This is a ``session``-scoped fixture which is called with ``(name, value)``. Example:

        .. code-block:: python

            def test_foo(record_testsuite_property):
                record_testsuite_property("ARCH", "PPC")
                record_testsuite_property("STORAGE_TYPE", "CEPH")

        :param name:
            The property name.
        :param value:
            The property value. Will be converted to a string.

        .. warning::

            Currently this fixture **does not work** with the
            `pytest-xdist <https://github.com/pytest-dev/pytest-xdist>`__ plugin. See
            :issue:`7767` for details.

    tmpdir_factory [session scope] -- .../_pytest/legacypath.py:303
        Return a :class:`pytest.TempdirFactory` instance for the test session.

    tmpdir -- .../_pytest/legacypath.py:310
        Return a temporary directory path object which is unique to each test
        function invocation, created as a sub directory of the base temporary
        directory.

        By default, a new base temporary directory is created each test session,
        and old bases are removed after 3 sessions, to aid in debugging. If
        ``--basetemp`` is used then it is cleared each session. See
        :ref:`temporary directory location and retention`.

        The returned object is a `legacy_path`_ object.

        .. note::
            These days, it is preferred to use ``tmp_path``.

            :ref:`About the tmpdir and tmpdir_factory fixtures<tmpdir and tmpdir_factory>`.

        .. _legacy_path: https://py.readthedocs.io/en/latest/path.html

    caplog -- .../_pytest/logging.py:602
        Access and control log capturing.

        Captured logs are available through the following properties/methods::

        * caplog.messages        -> list of format-interpolated log messages
        * caplog.text            -> string containing formatted log output
        * caplog.records         -> list of logging.LogRecord instances
        * caplog.record_tuples   -> list of (logger_name, level, message) tuples
        * caplog.clear()         -> clear captured records and formatted log output string

    monkeypatch -- .../_pytest/monkeypatch.py:33
        A convenient fixture for monkey-patching.

        The fixture provides these methods to modify objects, dictionaries, or
        :data:`os.environ`:

        * :meth:`monkeypatch.setattr(obj, name, value, raising=True) <pytest.MonkeyPatch.setattr>`
        * :meth:`monkeypatch.delattr(obj, name, raising=True) <pytest.MonkeyPatch.delattr>`
        * :meth:`monkeypatch.setitem(mapping, name, value) <pytest.MonkeyPatch.setitem>`
        * :meth:`monkeypatch.delitem(obj, name, raising=True) <pytest.MonkeyPatch.delitem>`
        * :meth:`monkeypatch.setenv(name, value, prepend=None) <pytest.MonkeyPatch.setenv>`
        * :meth:`monkeypatch.delenv(name, raising=True) <pytest.MonkeyPatch.delenv>`
        * :meth:`monkeypatch.syspath_prepend(path) <pytest.MonkeyPatch.syspath_prepend>`
        * :meth:`monkeypatch.chdir(path) <pytest.MonkeyPatch.chdir>`
        * :meth:`monkeypatch.context() <pytest.MonkeyPatch.context>`

        All modifications will be undone after the requesting test function or
        fixture has finished. The ``raising`` parameter determines if a :class:`KeyError`
        or :class:`AttributeError` will be raised if the set/deletion operation does not have the
        specified target.

        To undo modifications done by the fixture in a contained scope,
        use :meth:`context() <pytest.MonkeyPatch.context>`.

    recwarn -- .../_pytest/recwarn.py:32
        Return a :class:`WarningsRecorder` instance that records all warnings emitted by test functions.

        See https://docs.pytest.org/en/latest/how-to/capture-warnings.html for information
        on warning categories.

    tmp_path_factory [session scope] -- .../_pytest/tmpdir.py:242
        Return a :class:`pytest.TempPathFactory` instance for the test session.

    tmp_path -- .../_pytest/tmpdir.py:257
        Return a temporary directory path object which is unique to each test
        function invocation, created as a sub directory of the base temporary
        directory.

        By default, a new base temporary directory is created each test session,
        and old bases are removed after 3 sessions, to aid in debugging.
        This behavior can be configured with :confval:`tmp_path_retention_count` and
        :confval:`tmp_path_retention_policy`.
        If ``--basetemp`` is used then it is cleared each session. See
        :ref:`temporary directory location and retention`.

        The returned object is a :class:`pathlib.Path` object.


    ========================== no tests ran in 0.12s ===========================

You can also interactively ask for help, e.g. by typing on the Python interactive prompt something like:

.. code-block:: python

    import pytest

    help(pytest)
