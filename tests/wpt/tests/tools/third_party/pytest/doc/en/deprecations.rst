.. _deprecations:

Deprecations and Removals
=========================

This page lists all pytest features that are currently deprecated or have been removed in past major releases.
The objective is to give users a clear rationale why a certain feature has been removed, and what alternatives
should be used instead.

.. contents::
    :depth: 3
    :local:


Deprecated Features
-------------------

Below is a complete list of all pytest features which are considered deprecated. Using those features will issue
:class:`~pytest.PytestWarning` or subclasses, which can be filtered using :ref:`standard warning filters <warnings>`.


.. _import-or-skip-import-error:

``pytest.importorskip`` default behavior regarding :class:`ImportError`
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 8.2

Traditionally :func:`pytest.importorskip` will capture :class:`ImportError`, with the original intent being to skip
tests where a dependent module is not installed, for example testing with different dependencies.

However some packages might be installed in the system, but are not importable due to
some other issue, for example, a compilation error or a broken installation. In those cases :func:`pytest.importorskip`
would still silently skip the test, but more often than not users would like to see the unexpected
error so the underlying issue can be fixed.

In ``8.2`` the ``exc_type`` parameter has been added, giving users the ability of passing :class:`ModuleNotFoundError`
to skip tests only if the module cannot really be found, and not because of some other error.

Catching only :class:`ModuleNotFoundError` by default (and letting other errors propagate) would be the best solution,
however for backward compatibility, pytest will keep the existing behavior but raise an warning if:

1. The captured exception is of type :class:`ImportError`, and:
2. The user does not pass ``exc_type`` explicitly.

If the import attempt raises :class:`ModuleNotFoundError` (the usual case), then the module is skipped and no
warning is emitted.

This way, the usual cases will keep working the same way, while unexpected errors will now issue a warning, with
users being able to supress the warning by passing ``exc_type=ImportError`` explicitly.

In ``9.0``, the warning will turn into an error, and in ``9.1`` :func:`pytest.importorskip` will only capture
:class:`ModuleNotFoundError` by default and no warnings will be issued anymore -- but users can still capture
:class:`ImportError` by passing it to ``exc_type``.


.. _node-ctor-fspath-deprecation:

``fspath`` argument for Node constructors replaced with ``pathlib.Path``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 7.0

In order to support the transition from ``py.path.local`` to :mod:`pathlib`,
the ``fspath`` argument to :class:`~_pytest.nodes.Node` constructors like
:func:`pytest.Function.from_parent()` and :func:`pytest.Class.from_parent()`
is now deprecated.

Plugins which construct nodes should pass the ``path`` argument, of type
:class:`pathlib.Path`, instead of the ``fspath`` argument.

Plugins which implement custom items and collectors are encouraged to replace
``fspath`` parameters (``py.path.local``) with ``path`` parameters
(``pathlib.Path``), and drop any other usage of the ``py`` library if possible.

If possible, plugins with custom items should use :ref:`cooperative
constructors <uncooperative-constructors-deprecated>` to avoid hardcoding
arguments they only pass on to the superclass.

.. note::
    The name of the :class:`~_pytest.nodes.Node` arguments and attributes (the
    new attribute being ``path``) is **the opposite** of the situation for
    hooks, :ref:`outlined below <legacy-path-hooks-deprecated>` (the old
    argument being ``path``).

    This is an unfortunate artifact due to historical reasons, which should be
    resolved in future versions as we slowly get rid of the :pypi:`py`
    dependency (see :issue:`9283` for a longer discussion).

Due to the ongoing migration of methods like :meth:`~pytest.Item.reportinfo`
which still is expected to return a ``py.path.local`` object, nodes still have
both ``fspath`` (``py.path.local``) and ``path`` (``pathlib.Path``) attributes,
no matter what argument was used in the constructor. We expect to deprecate the
``fspath`` attribute in a future release.


Configuring hook specs/impls using markers
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Before pluggy, pytest's plugin library, was its own package and had a clear API,
pytest just used ``pytest.mark`` to configure hooks.

The :py:func:`pytest.hookimpl` and :py:func:`pytest.hookspec` decorators
have been available since years and should be used instead.

.. code-block:: python

    @pytest.mark.tryfirst
    def pytest_runtest_call(): ...


    # or
    def pytest_runtest_call(): ...


    pytest_runtest_call.tryfirst = True

should be changed to:

.. code-block:: python

    @pytest.hookimpl(tryfirst=True)
    def pytest_runtest_call(): ...

Changed ``hookimpl`` attributes:

* ``tryfirst``
* ``trylast``
* ``optionalhook``
* ``hookwrapper``

Changed ``hookwrapper`` attributes:

* ``firstresult``
* ``historic``


.. _legacy-path-hooks-deprecated:

``py.path.local`` arguments for hooks replaced with ``pathlib.Path``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 7.0

In order to support the transition from ``py.path.local`` to :mod:`pathlib`, the following hooks now receive additional arguments:

*  :hook:`pytest_ignore_collect(collection_path: pathlib.Path) <pytest_ignore_collect>` as equivalent to ``path``
*  :hook:`pytest_collect_file(file_path: pathlib.Path) <pytest_collect_file>` as equivalent to ``path``
*  :hook:`pytest_pycollect_makemodule(module_path: pathlib.Path) <pytest_pycollect_makemodule>` as equivalent to ``path``
*  :hook:`pytest_report_header(start_path: pathlib.Path) <pytest_report_header>` as equivalent to ``startdir``
*  :hook:`pytest_report_collectionfinish(start_path: pathlib.Path) <pytest_report_collectionfinish>` as equivalent to ``startdir``

The accompanying ``py.path.local`` based paths have been deprecated: plugins which manually invoke those hooks should only pass the new ``pathlib.Path`` arguments, and users should change their hook implementations to use the new ``pathlib.Path`` arguments.

.. note::
    The name of the :class:`~_pytest.nodes.Node` arguments and attributes,
    :ref:`outlined above <node-ctor-fspath-deprecation>` (the new attribute
    being ``path``) is **the opposite** of the situation for hooks (the old
    argument being ``path``).

    This is an unfortunate artifact due to historical reasons, which should be
    resolved in future versions as we slowly get rid of the :pypi:`py`
    dependency (see :issue:`9283` for a longer discussion).

Directly constructing internal classes
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 7.0

Directly constructing the following classes is now deprecated:

- ``_pytest.mark.structures.Mark``
- ``_pytest.mark.structures.MarkDecorator``
- ``_pytest.mark.structures.MarkGenerator``
- ``_pytest.python.Metafunc``
- ``_pytest.runner.CallInfo``
- ``_pytest._code.ExceptionInfo``
- ``_pytest.config.argparsing.Parser``
- ``_pytest.config.argparsing.OptionGroup``
- ``_pytest.pytester.HookRecorder``

These constructors have always been considered private, but now issue a deprecation warning, which may become a hard error in pytest 8.

.. _diamond-inheritance-deprecated:

Diamond inheritance between :class:`pytest.Collector` and :class:`pytest.Item`
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 7.0

Defining a custom pytest node type which is both an :class:`~pytest.Item` and a :class:`~pytest.Collector` (e.g. :class:`~pytest.File`) now issues a warning.
It was never sanely supported and triggers hard to debug errors.

Some plugins providing linting/code analysis have been using this as a hack.
Instead, a separate collector node should be used, which collects the item. See
:ref:`non-python tests` for an example, as well as an `example pr fixing inheritance`_.

.. _example pr fixing inheritance: https://github.com/asmeurer/pytest-flakes/pull/40/files


.. _uncooperative-constructors-deprecated:

Constructors of custom :class:`~_pytest.nodes.Node` subclasses should take ``**kwargs``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 7.0

If custom subclasses of nodes like :class:`pytest.Item` override the
``__init__`` method, they should take ``**kwargs``. Thus,

.. code-block:: python

    class CustomItem(pytest.Item):
        def __init__(self, name, parent, additional_arg):
            super().__init__(name, parent)
            self.additional_arg = additional_arg

should be turned into:

.. code-block:: python

    class CustomItem(pytest.Item):
        def __init__(self, *, additional_arg, **kwargs):
            super().__init__(**kwargs)
            self.additional_arg = additional_arg

to avoid hard-coding the arguments pytest can pass to the superclass.
See :ref:`non-python tests` for a full example.

For cases without conflicts, no deprecation warning is emitted. For cases with
conflicts (such as :class:`pytest.File` now taking ``path`` instead of
``fspath``, as :ref:`outlined above <node-ctor-fspath-deprecation>`), a
deprecation warning is now raised.

Applying a mark to a fixture function
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 7.4

Applying a mark to a fixture function never had any effect, but it is a common user error.

.. code-block:: python

    @pytest.mark.usefixtures("clean_database")
    @pytest.fixture
    def user() -> User: ...

Users expected in this case that the ``usefixtures`` mark would have its intended effect of using the ``clean_database`` fixture when ``user`` was invoked, when in fact it has no effect at all.

Now pytest will issue a warning when it encounters this problem, and will raise an error in the future versions.


Returning non-None value in test functions
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 7.2

A :class:`pytest.PytestReturnNotNoneWarning` is now emitted if a test function returns something other than `None`.

This prevents a common mistake among beginners that expect that returning a `bool` would cause a test to pass or fail, for example:

.. code-block:: python

    @pytest.mark.parametrize(
        ["a", "b", "result"],
        [
            [1, 2, 5],
            [2, 3, 8],
            [5, 3, 18],
        ],
    )
    def test_foo(a, b, result):
        return foo(a, b) == result

Given that pytest ignores the return value, this might be surprising that it will never fail.

The proper fix is to change the `return` to an `assert`:

.. code-block:: python

    @pytest.mark.parametrize(
        ["a", "b", "result"],
        [
            [1, 2, 5],
            [2, 3, 8],
            [5, 3, 18],
        ],
    )
    def test_foo(a, b, result):
        assert foo(a, b) == result


The ``yield_fixture`` function/decorator
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 6.2

``pytest.yield_fixture`` is a deprecated alias for :func:`pytest.fixture`.

It has been so for a very long time, so can be search/replaced safely.


Removed Features and Breaking Changes
-------------------------------------

As stated in our :ref:`backwards-compatibility` policy, deprecated features are removed only in major releases after
an appropriate period of deprecation has passed.

Some breaking changes which could not be deprecated are also listed.

.. _nose-deprecation:

Support for tests written for nose
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 7.2
.. versionremoved:: 8.0

Support for running tests written for `nose <https://nose.readthedocs.io/en/latest/>`__ is now deprecated.

``nose`` has been in maintenance mode-only for years, and maintaining the plugin is not trivial as it spills
over the code base (see :issue:`9886` for more details).

setup/teardown
^^^^^^^^^^^^^^

One thing that might catch users by surprise is that plain ``setup`` and ``teardown`` methods are not pytest native,
they are in fact part of the ``nose`` support.


.. code-block:: python

    class Test:
        def setup(self):
            self.resource = make_resource()

        def teardown(self):
            self.resource.close()

        def test_foo(self): ...

        def test_bar(self): ...



Native pytest support uses ``setup_method`` and ``teardown_method`` (see :ref:`xunit-method-setup`), so the above should be changed to:

.. code-block:: python

    class Test:
        def setup_method(self):
            self.resource = make_resource()

        def teardown_method(self):
            self.resource.close()

        def test_foo(self): ...

        def test_bar(self): ...


This is easy to do in an entire code base by doing a simple find/replace.

@with_setup
^^^^^^^^^^^

Code using `@with_setup <with-setup-nose>`_ such as this:

.. code-block:: python

    from nose.tools import with_setup


    def setup_some_resource(): ...


    def teardown_some_resource(): ...


    @with_setup(setup_some_resource, teardown_some_resource)
    def test_foo(): ...

Will also need to be ported to a supported pytest style. One way to do it is using a fixture:

.. code-block:: python

    import pytest


    def setup_some_resource(): ...


    def teardown_some_resource(): ...


    @pytest.fixture
    def some_resource():
        setup_some_resource()
        yield
        teardown_some_resource()


    def test_foo(some_resource): ...


.. _`with-setup-nose`: https://nose.readthedocs.io/en/latest/testing_tools.html?highlight=with_setup#nose.tools.with_setup


The ``compat_co_firstlineno`` attribute
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Nose inspects this attribute on function objects to allow overriding the function's inferred line number.
Pytest no longer respects this attribute.



Passing ``msg=`` to ``pytest.skip``, ``pytest.fail`` or ``pytest.exit``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 7.0
.. versionremoved:: 8.0

Passing the keyword argument ``msg`` to :func:`pytest.skip`, :func:`pytest.fail` or :func:`pytest.exit`
is now deprecated and ``reason`` should be used instead.  This change is to bring consistency between these
functions and the ``@pytest.mark.skip`` and ``@pytest.mark.xfail`` markers which already accept a ``reason`` argument.

.. code-block:: python

    def test_fail_example():
        # old
        pytest.fail(msg="foo")
        # new
        pytest.fail(reason="bar")


    def test_skip_example():
        # old
        pytest.skip(msg="foo")
        # new
        pytest.skip(reason="bar")


    def test_exit_example():
        # old
        pytest.exit(msg="foo")
        # new
        pytest.exit(reason="bar")


.. _instance-collector-deprecation:

The ``pytest.Instance`` collector
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 7.0

The ``pytest.Instance`` collector type has been removed.

Previously, Python test methods were collected as :class:`~pytest.Class` -> ``Instance`` -> :class:`~pytest.Function`.
Now :class:`~pytest.Class` collects the test methods directly.

Most plugins which reference ``Instance`` do so in order to ignore or skip it,
using a check such as ``if isinstance(node, Instance): return``.
Such plugins should simply remove consideration of ``Instance`` on pytest>=7.
However, to keep such uses working, a dummy type has been instanced in ``pytest.Instance`` and ``_pytest.python.Instance``,
and importing it emits a deprecation warning. This was removed in pytest 8.


Using ``pytest.warns(None)``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 7.0
.. versionremoved:: 8.0

:func:`pytest.warns(None) <pytest.warns>` is now deprecated because it was frequently misused.
Its correct usage was checking that the code emits at least one warning of any type - like ``pytest.warns()``
or ``pytest.warns(Warning)``.

See :ref:`warns use cases` for examples.


Backward compatibilities in ``Parser.addoption``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 2.4
.. versionremoved:: 8.0

Several behaviors of :meth:`Parser.addoption <pytest.Parser.addoption>` are now
removed in pytest 8 (deprecated since pytest 2.4.0):

- ``parser.addoption(..., help=".. %default ..")`` - use ``%(default)s`` instead.
- ``parser.addoption(..., type="int/string/float/complex")`` - use ``type=int`` etc. instead.


The ``--strict`` command-line option
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 6.2
.. versionremoved:: 8.0

The ``--strict`` command-line option has been deprecated in favor of ``--strict-markers``, which
better conveys what the option does.

We have plans to maybe in the future to reintroduce ``--strict`` and make it an encompassing
flag for all strictness related options (``--strict-markers`` and ``--strict-config``
at the moment, more might be introduced in the future).


.. _cmdline-preparse-deprecated:

Implementing the ``pytest_cmdline_preparse`` hook
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 7.0
.. versionremoved:: 8.0

Implementing the ``pytest_cmdline_preparse`` hook has been officially deprecated.
Implement the :hook:`pytest_load_initial_conftests` hook instead.

.. code-block:: python

    def pytest_cmdline_preparse(config: Config, args: List[str]) -> None: ...


    # becomes:


    def pytest_load_initial_conftests(
        early_config: Config, parser: Parser, args: List[str]
    ) -> None: ...


Collection changes in pytest 8
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Added a new :class:`pytest.Directory` base collection node, which all collector nodes for filesystem directories are expected to subclass.
This is analogous to the existing :class:`pytest.File` for file nodes.

Changed :class:`pytest.Package` to be a subclass of :class:`pytest.Directory`.
A ``Package`` represents a filesystem directory which is a Python package,
i.e. contains an ``__init__.py`` file.

:class:`pytest.Package` now only collects files in its own directory; previously it collected recursively.
Sub-directories are collected as sub-collector nodes, thus creating a collection tree which mirrors the filesystem hierarchy.

:attr:`session.name <pytest.Session.name>` is now ``""``; previously it was the rootdir directory name.
This matches :attr:`session.nodeid <_pytest.nodes.Node.nodeid>` which has always been `""`.

Added a new :class:`pytest.Dir` concrete collection node, a subclass of :class:`pytest.Directory`.
This node represents a filesystem directory, which is not a :class:`pytest.Package`,
i.e. does not contain an ``__init__.py`` file.
Similarly to ``Package``, it only collects the files in its own directory,
while collecting sub-directories as sub-collector nodes.

Files and directories are now collected in alphabetical order jointly, unless changed by a plugin.
Previously, files were collected before directories.

The collection tree now contains directories/packages up to the :ref:`rootdir <rootdir>`,
for initial arguments that are found within the rootdir.
For files outside the rootdir, only the immediate directory/package is collected --
note however that collecting from outside the rootdir is discouraged.

As an example, given the following filesystem tree::

    myroot/
        pytest.ini
        top/
        ├── aaa
        │   └── test_aaa.py
        ├── test_a.py
        ├── test_b
        │   ├── __init__.py
        │   └── test_b.py
        ├── test_c.py
        └── zzz
            ├── __init__.py
            └── test_zzz.py

the collection tree, as shown by `pytest --collect-only top/` but with the otherwise-hidden :class:`~pytest.Session` node added for clarity,
is now the following::

    <Session>
      <Dir myroot>
        <Dir top>
          <Dir aaa>
            <Module test_aaa.py>
              <Function test_it>
          <Module test_a.py>
            <Function test_it>
          <Package test_b>
            <Module test_b.py>
              <Function test_it>
          <Module test_c.py>
            <Function test_it>
          <Package zzz>
            <Module test_zzz.py>
              <Function test_it>

Previously, it was::

    <Session>
      <Module top/test_a.py>
        <Function test_it>
      <Module top/test_c.py>
        <Function test_it>
      <Module top/aaa/test_aaa.py>
        <Function test_it>
      <Package test_b>
        <Module test_b.py>
          <Function test_it>
      <Package zzz>
        <Module test_zzz.py>
          <Function test_it>

Code/plugins which rely on a specific shape of the collection tree might need to update.


:class:`pytest.Package` is no longer a :class:`pytest.Module` or :class:`pytest.File`
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionchanged:: 8.0

The ``Package`` collector node designates a Python package, that is, a directory with an `__init__.py` file.
Previously ``Package`` was a subtype of ``pytest.Module`` (which represents a single Python module),
the module being the `__init__.py` file.
This has been deemed a design mistake (see :issue:`11137` and :issue:`7777` for details).

The ``path`` property of ``Package`` nodes now points to the package directory instead of the ``__init__.py`` file.

Note that a ``Module`` node for ``__init__.py`` (which is not a ``Package``) may still exist,
if it is picked up during collection (e.g. if you configured :confval:`python_files` to include ``__init__.py`` files).


Collecting ``__init__.py`` files no longer collects package
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 8.0

Running `pytest pkg/__init__.py` now collects the `pkg/__init__.py` file (module) only.
Previously, it collected the entire `pkg` package, including other test files in the directory, but excluding tests in the `__init__.py` file itself
(unless :confval:`python_files` was changed to allow `__init__.py` file).

To collect the entire package, specify just the directory: `pytest pkg`.


The ``pytest.collect`` module
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 6.0
.. versionremoved:: 7.0

The ``pytest.collect`` module is no longer part of the public API, all its names
should now be imported from ``pytest`` directly instead.



The ``pytest_warning_captured`` hook
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 6.0
.. versionremoved:: 7.0

This hook has an `item` parameter which cannot be serialized by ``pytest-xdist``.

Use the ``pytest_warning_recorded`` hook instead, which replaces the ``item`` parameter
by a ``nodeid`` parameter.



The ``pytest._fillfuncargs`` function
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 6.0
.. versionremoved:: 7.0

This function was kept for backward compatibility with an older plugin.

It's functionality is not meant to be used directly, but if you must replace
it, use `function._request._fillfixtures()` instead, though note this is not
a public API and may break in the future.


``--no-print-logs`` command-line option
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 5.4
.. versionremoved:: 6.0


The ``--no-print-logs`` option and ``log_print`` ini setting are removed. If
you used them, please use ``--show-capture`` instead.

A ``--show-capture`` command-line option was added in ``pytest 3.5.0`` which allows to specify how to
display captured output when tests fail: ``no``, ``stdout``, ``stderr``, ``log`` or ``all`` (the default).


.. _resultlog deprecated:

Result log (``--result-log``)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. deprecated:: 4.0
.. versionremoved:: 6.0

The ``--result-log`` option produces a stream of test reports which can be
analysed at runtime, but it uses a custom format which requires users to implement their own
parser.

The  `pytest-reportlog <https://github.com/pytest-dev/pytest-reportlog>`__ plugin provides a ``--report-log`` option, a more standard and extensible alternative, producing
one JSON object per-line, and should cover the same use cases. Please try it out and provide feedback.

The ``pytest-reportlog`` plugin might even be merged into the core
at some point, depending on the plans for the plugins and number of users using it.

``pytest_collect_directory`` hook
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 6.0

The ``pytest_collect_directory`` hook has not worked properly for years (it was called
but the results were ignored). Users may consider using :hook:`pytest_collection_modifyitems` instead.

TerminalReporter.writer
~~~~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 6.0

The ``TerminalReporter.writer`` attribute has been deprecated and should no longer be used. This
was inadvertently exposed as part of the public API of that plugin and ties it too much
with ``py.io.TerminalWriter``.

Plugins that used ``TerminalReporter.writer`` directly should instead use ``TerminalReporter``
methods that provide the same functionality.

.. _junit-family changed default value:

``junit_family`` default value change to "xunit2"
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionchanged:: 6.0

The default value of ``junit_family`` option will change to ``xunit2`` in pytest 6.0, which
is an update of the old ``xunit1`` format and is supported by default in modern tools
that manipulate this type of file (for example, Jenkins, Azure Pipelines, etc.).

Users are recommended to try the new ``xunit2`` format and see if their tooling that consumes the JUnit
XML file supports it.

To use the new format, update your ``pytest.ini``:

.. code-block:: ini

    [pytest]
    junit_family=xunit2

If you discover that your tooling does not support the new format, and want to keep using the
legacy version, set the option to ``legacy`` instead:

.. code-block:: ini

    [pytest]
    junit_family=legacy

By using ``legacy`` you will keep using the legacy/xunit1 format when upgrading to
pytest 6.0, where the default format will be ``xunit2``.

In order to let users know about the transition, pytest will issue a warning in case
the ``--junit-xml`` option is given in the command line but ``junit_family`` is not explicitly
configured in ``pytest.ini``.

Services known to support the ``xunit2`` format:

* `Jenkins <https://www.jenkins.io/>`__ with the `JUnit <https://plugins.jenkins.io/junit>`__ plugin.
* `Azure Pipelines <https://azure.microsoft.com/en-us/services/devops/pipelines>`__.

Node Construction changed to ``Node.from_parent``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionchanged:: 6.0

The construction of nodes now should use the named constructor ``from_parent``.
This limitation in api surface intends to enable better/simpler refactoring of the collection tree.

This means that instead of :code:`MyItem(name="foo", parent=collector, obj=42)`
one now has to invoke :code:`MyItem.from_parent(collector, name="foo")`.

Plugins that wish to support older versions of pytest and suppress the warning can use
`hasattr` to check if `from_parent` exists in that version:

.. code-block:: python

    def pytest_pycollect_makeitem(collector, name, obj):
        if hasattr(MyItem, "from_parent"):
            item = MyItem.from_parent(collector, name="foo")
            item.obj = 42
            return item
        else:
            return MyItem(name="foo", parent=collector, obj=42)

Note that ``from_parent`` should only be called with keyword arguments for the parameters.


``pytest.fixture`` arguments are keyword only
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 6.0

Passing arguments to pytest.fixture() as positional arguments has been removed - pass them by keyword instead.

``funcargnames`` alias for ``fixturenames``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 6.0

The ``FixtureRequest``, ``Metafunc``, and ``Function`` classes track the names of
their associated fixtures, with the aptly-named ``fixturenames`` attribute.

Prior to pytest 2.3, this attribute was named ``funcargnames``, and we have kept
that as an alias since.  It is finally due for removal, as it is often confusing
in places where we or plugin authors must distinguish between fixture names and
names supplied by non-fixture things such as ``pytest.mark.parametrize``.


.. _pytest.config global deprecated:

``pytest.config`` global
~~~~~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 5.0

The ``pytest.config`` global object is deprecated.  Instead use
``request.config`` (via the ``request`` fixture) or if you are a plugin author
use the ``pytest_configure(config)`` hook. Note that many hooks can also access
the ``config`` object indirectly, through ``session.config`` or ``item.config`` for example.


.. _`raises message deprecated`:

``"message"`` parameter of ``pytest.raises``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 5.0

It is a common mistake to think this parameter will match the exception message, while in fact
it only serves to provide a custom message in case the ``pytest.raises`` check fails. To prevent
users from making this mistake, and because it is believed to be little used, pytest is
deprecating it without providing an alternative for the moment.

If you have a valid use case for this parameter, consider that to obtain the same results
you can just call ``pytest.fail`` manually at the end of the ``with`` statement.

For example:

.. code-block:: python

    with pytest.raises(TimeoutError, message="Client got unexpected message"):
        wait_for(websocket.recv(), 0.5)


Becomes:

.. code-block:: python

    with pytest.raises(TimeoutError):
        wait_for(websocket.recv(), 0.5)
        pytest.fail("Client got unexpected message")


If you still have concerns about this deprecation and future removal, please comment on
:issue:`3974`.


.. _raises-warns-exec:

``raises`` / ``warns`` with a string as the second argument
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 5.0

Use the context manager form of these instead.  When necessary, invoke ``exec``
directly.

Example:

.. code-block:: python

    pytest.raises(ZeroDivisionError, "1 / 0")
    pytest.raises(SyntaxError, "a $ b")

    pytest.warns(DeprecationWarning, "my_function()")
    pytest.warns(SyntaxWarning, "assert(1, 2)")

Becomes:

.. code-block:: python

    with pytest.raises(ZeroDivisionError):
        1 / 0
    with pytest.raises(SyntaxError):
        exec("a $ b")  # exec is required for invalid syntax

    with pytest.warns(DeprecationWarning):
        my_function()
    with pytest.warns(SyntaxWarning):
        exec("assert(1, 2)")  # exec is used to avoid a top-level warning




Using ``Class`` in custom Collectors
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 4.0

Using objects named ``"Class"`` as a way to customize the type of nodes that are collected in ``Collector``
subclasses has been deprecated. Users instead should use ``pytest_pycollect_makeitem`` to customize node types during
collection.

This issue should affect only advanced plugins who create new collection types, so if you see this warning
message please contact the authors so they can change the code.


.. _marks in pytest.parametrize deprecated:

marks in ``pytest.mark.parametrize``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 4.0

Applying marks to values of a ``pytest.mark.parametrize`` call is now deprecated. For example:

.. code-block:: python

    @pytest.mark.parametrize(
        "a, b",
        [
            (3, 9),
            pytest.mark.xfail(reason="flaky")(6, 36),
            (10, 100),
            (20, 200),
            (40, 400),
            (50, 500),
        ],
    )
    def test_foo(a, b): ...

This code applies the ``pytest.mark.xfail(reason="flaky")`` mark to the ``(6, 36)`` value of the above parametrization
call.

This was considered hard to read and understand, and also its implementation presented problems to the code preventing
further internal improvements in the marks architecture.

To update the code, use ``pytest.param``:

.. code-block:: python

    @pytest.mark.parametrize(
        "a, b",
        [
            (3, 9),
            pytest.param(6, 36, marks=pytest.mark.xfail(reason="flaky")),
            (10, 100),
            (20, 200),
            (40, 400),
            (50, 500),
        ],
    )
    def test_foo(a, b): ...


.. _pytest_funcarg__ prefix deprecated:

``pytest_funcarg__`` prefix
~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 4.0

In very early pytest versions fixtures could be defined using the ``pytest_funcarg__`` prefix:

.. code-block:: python

    def pytest_funcarg__data():
        return SomeData()

Switch over to the ``@pytest.fixture`` decorator:

.. code-block:: python

    @pytest.fixture
    def data():
        return SomeData()



[pytest] section in setup.cfg files
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 4.0

``[pytest]`` sections in ``setup.cfg`` files should now be named ``[tool:pytest]``
to avoid conflicts with other distutils commands.


.. _metafunc.addcall deprecated:

Metafunc.addcall
~~~~~~~~~~~~~~~~

.. versionremoved:: 4.0

``Metafunc.addcall`` was a precursor to the current parametrized mechanism. Users should use
:meth:`pytest.Metafunc.parametrize` instead.

Example:

.. code-block:: python

    def pytest_generate_tests(metafunc):
        metafunc.addcall({"i": 1}, id="1")
        metafunc.addcall({"i": 2}, id="2")

Becomes:

.. code-block:: python

    def pytest_generate_tests(metafunc):
        metafunc.parametrize("i", [1, 2], ids=["1", "2"])


.. _cached_setup deprecated:

``cached_setup``
~~~~~~~~~~~~~~~~

.. versionremoved:: 4.0

``request.cached_setup`` was the precursor of the setup/teardown mechanism available to fixtures.

Example:

.. code-block:: python

    @pytest.fixture
    def db_session():
        return request.cached_setup(
            setup=Session.create, teardown=lambda session: session.close(), scope="module"
        )

This should be updated to make use of standard fixture mechanisms:

.. code-block:: python

    @pytest.fixture(scope="module")
    def db_session():
        session = Session.create()
        yield session
        session.close()


You can consult :std:doc:`funcarg comparison section in the docs <funcarg_compare>` for
more information.


.. _pytest_plugins in non-top-level conftest files deprecated:

pytest_plugins in non-top-level conftest files
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 4.0

Defining :globalvar:`pytest_plugins` is now deprecated in non-top-level conftest.py
files because they will activate referenced plugins *globally*, which is surprising because for all other pytest
features ``conftest.py`` files are only *active* for tests at or below it.


.. _config.warn and node.warn deprecated:

``Config.warn`` and ``Node.warn``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 4.0

Those methods were part of the internal pytest warnings system, but since ``3.8`` pytest is using the builtin warning
system for its own warnings, so those two functions are now deprecated.

``Config.warn`` should be replaced by calls to the standard ``warnings.warn``, example:

.. code-block:: python

    config.warn("C1", "some warning")

Becomes:

.. code-block:: python

    warnings.warn(pytest.PytestWarning("some warning"))

``Node.warn`` now supports two signatures:

* ``node.warn(PytestWarning("some message"))``: is now the **recommended** way to call this function.
  The warning instance must be a PytestWarning or subclass.

* ``node.warn("CI", "some message")``: this code/message form has been **removed** and should be converted to the warning instance form above.

.. _record_xml_property deprecated:

record_xml_property
~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 4.0

The ``record_xml_property`` fixture is now deprecated in favor of the more generic ``record_property``, which
can be used by other consumers (for example ``pytest-html``) to obtain custom information about the test run.

This is just a matter of renaming the fixture as the API is the same:

.. code-block:: python

    def test_foo(record_xml_property): ...

Change to:

.. code-block:: python

    def test_foo(record_property): ...


.. _passing command-line string to pytest.main deprecated:

Passing command-line string to ``pytest.main()``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 4.0

Passing a command-line string to ``pytest.main()`` is deprecated:

.. code-block:: python

    pytest.main("-v -s")

Pass a list instead:

.. code-block:: python

    pytest.main(["-v", "-s"])


By passing a string, users expect that pytest will interpret that command-line using the shell rules they are working
on (for example ``bash`` or ``Powershell``), but this is very hard/impossible to do in a portable way.


.. _calling fixtures directly deprecated:

Calling fixtures directly
~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 4.0

Calling a fixture function directly, as opposed to request them in a test function, is deprecated.

For example:

.. code-block:: python

    @pytest.fixture
    def cell():
        return ...


    @pytest.fixture
    def full_cell():
        cell = cell()
        cell.make_full()
        return cell

This is a great source of confusion to new users, which will often call the fixture functions and request them from test functions interchangeably, which breaks the fixture resolution model.

In those cases just request the function directly in the dependent fixture:

.. code-block:: python

    @pytest.fixture
    def cell():
        return ...


    @pytest.fixture
    def full_cell(cell):
        cell.make_full()
        return cell

Alternatively if the fixture function is called multiple times inside a test (making it hard to apply the above pattern) or
if you would like to make minimal changes to the code, you can create a fixture which calls the original function together
with the ``name`` parameter:

.. code-block:: python

    def cell():
        return ...


    @pytest.fixture(name="cell")
    def cell_fixture():
        return cell()


.. _yield tests deprecated:

``yield`` tests
~~~~~~~~~~~~~~~

.. versionremoved:: 4.0

pytest supported ``yield``-style tests, where a test function actually ``yield`` functions and values
that are then turned into proper test methods. Example:

.. code-block:: python

    def check(x, y):
        assert x**x == y


    def test_squared():
        yield check, 2, 4
        yield check, 3, 9

This would result into two actual test functions being generated.

This form of test function doesn't support fixtures properly, and users should switch to ``pytest.mark.parametrize``:

.. code-block:: python

    @pytest.mark.parametrize("x, y", [(2, 4), (3, 9)])
    def test_squared(x, y):
        assert x**x == y

.. _internal classes accessed through node deprecated:

Internal classes accessed through ``Node``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 4.0

Access of ``Module``, ``Function``, ``Class``, ``Instance``, ``File`` and ``Item`` through ``Node`` instances now issue
this warning:

.. code-block:: text

    usage of Function.Module is deprecated, please use pytest.Module instead

Users should just ``import pytest`` and access those objects using the ``pytest`` module.

This has been documented as deprecated for years, but only now we are actually emitting deprecation warnings.

``Node.get_marker``
~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 4.0

As part of a large :ref:`marker-revamp`, ``_pytest.nodes.Node.get_marker`` is removed. See
:ref:`the documentation <update marker code>` on tips on how to update your code.


``somefunction.markname``
~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 4.0

As part of a large :ref:`marker-revamp` we already deprecated using ``MarkInfo``
the only correct way to get markers of an element is via ``node.iter_markers(name)``.


.. _pytest.namespace deprecated:

``pytest_namespace``
~~~~~~~~~~~~~~~~~~~~

.. versionremoved:: 4.0

This hook is deprecated because it greatly complicates the pytest internals regarding configuration and initialization, making some
bug fixes and refactorings impossible.

Example of usage:

.. code-block:: python

    class MySymbol: ...


    def pytest_namespace():
        return {"my_symbol": MySymbol()}


Plugin authors relying on this hook should instead require that users now import the plugin modules directly (with an appropriate public API).

As a stopgap measure, plugin authors may still inject their names into pytest's namespace, usually during ``pytest_configure``:

.. code-block:: python

    import pytest


    def pytest_configure():
        pytest.my_symbol = MySymbol()
