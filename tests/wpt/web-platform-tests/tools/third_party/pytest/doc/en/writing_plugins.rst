.. _plugins:
.. _`writing-plugins`:

Writing plugins
===============

It is easy to implement `local conftest plugins`_ for your own project
or `pip-installable plugins`_ that can be used throughout many projects,
including third party projects.  Please refer to :ref:`using plugins` if you
only want to use but not write plugins.

A plugin contains one or multiple hook functions. :ref:`Writing hooks <writinghooks>`
explains the basics and details of how you can write a hook function yourself.
``pytest`` implements all aspects of configuration, collection, running and
reporting by calling `well specified hooks`_ of the following plugins:

* :ref:`builtin plugins`: loaded from pytest's internal ``_pytest`` directory.

* :ref:`external plugins <extplugins>`: modules discovered through
  `setuptools entry points`_

* `conftest.py plugins`_: modules auto-discovered in test directories

In principle, each hook call is a ``1:N`` Python function call where ``N`` is the
number of registered implementation functions for a given specification.
All specifications and implementations follow the ``pytest_`` prefix
naming convention, making them easy to distinguish and find.

.. _`pluginorder`:

Plugin discovery order at tool startup
--------------------------------------

``pytest`` loads plugin modules at tool startup in the following way:

* by loading all builtin plugins

* by loading all plugins registered through `setuptools entry points`_.

* by pre-scanning the command line for the ``-p name`` option
  and loading the specified plugin before actual command line parsing.

* by loading all :file:`conftest.py` files as inferred by the command line
  invocation:

  - if no test paths are specified use current dir as a test path
  - if exists, load ``conftest.py`` and ``test*/conftest.py`` relative
    to the directory part of the first test path.

  Note that pytest does not find ``conftest.py`` files in deeper nested
  sub directories at tool startup.  It is usually a good idea to keep
  your ``conftest.py`` file in the top level test or project root directory.

* by recursively loading all plugins specified by the
  ``pytest_plugins`` variable in ``conftest.py`` files


.. _`pytest/plugin`: http://bitbucket.org/pytest-dev/pytest/src/tip/pytest/plugin/
.. _`conftest.py plugins`:
.. _`localplugin`:
.. _`local conftest plugins`:

conftest.py: local per-directory plugins
----------------------------------------

Local ``conftest.py`` plugins contain directory-specific hook
implementations.  Hook Session and test running activities will
invoke all hooks defined in ``conftest.py`` files closer to the
root of the filesystem.  Example of implementing the
``pytest_runtest_setup`` hook so that is called for tests in the ``a``
sub directory but not for other directories::

    a/conftest.py:
        def pytest_runtest_setup(item):
            # called for running each test in 'a' directory
            print ("setting up", item)

    a/test_sub.py:
        def test_sub():
            pass

    test_flat.py:
        def test_flat():
            pass

Here is how you might run it::

     pytest test_flat.py   # will not show "setting up"
     pytest a/test_sub.py  # will show "setting up"

.. note::
    If you have ``conftest.py`` files which do not reside in a
    python package directory (i.e. one containing an ``__init__.py``) then
    "import conftest" can be ambiguous because there might be other
    ``conftest.py`` files as well on your ``PYTHONPATH`` or ``sys.path``.
    It is thus good practice for projects to either put ``conftest.py``
    under a package scope or to never import anything from a
    ``conftest.py`` file.

    See also: :ref:`pythonpath`.


Writing your own plugin
-----------------------

.. _`setuptools`: http://pypi.python.org/pypi/setuptools

If you want to write a plugin, there are many real-life examples
you can copy from:

* a custom collection example plugin: :ref:`yaml plugin`
* around 20 :ref:`builtin plugins` which provide pytest's own functionality
* many `external plugins <http://plugincompat.herokuapp.com>`_ providing additional features

All of these plugins implement the documented `well specified hooks`_
to extend and add functionality.

.. note::
    Make sure to check out the excellent
    `cookiecutter-pytest-plugin <https://github.com/pytest-dev/cookiecutter-pytest-plugin>`_
    project, which is a `cookiecutter template <https://github.com/audreyr/cookiecutter>`_
    for authoring plugins.

    The template provides an excellent starting point with a working plugin,
    tests running with tox, a comprehensive README file as well as a
    pre-configured entry-point.

Also consider :ref:`contributing your plugin to pytest-dev<submitplugin>`
once it has some happy users other than yourself.


.. _`setuptools entry points`:
.. _`pip-installable plugins`:

Making your plugin installable by others
----------------------------------------

If you want to make your plugin externally available, you
may define a so-called entry point for your distribution so
that ``pytest`` finds your plugin module.  Entry points are
a feature that is provided by `setuptools`_. pytest looks up
the ``pytest11`` entrypoint to discover its
plugins and you can thus make your plugin available by defining
it in your setuptools-invocation:

.. sourcecode:: python

    # sample ./setup.py file
    from setuptools import setup

    setup(
        name="myproject",
        packages = ['myproject']

        # the following makes a plugin available to pytest
        entry_points = {
            'pytest11': [
                'name_of_plugin = myproject.pluginmodule',
            ]
        },

        # custom PyPI classifier for pytest plugins
        classifiers=[
            "Framework :: Pytest",
        ],
    )

If a package is installed this way, ``pytest`` will load
``myproject.pluginmodule`` as a plugin which can define
`well specified hooks`_.

.. note::

    Make sure to include ``Framework :: Pytest`` in your list of
    `PyPI classifiers <https://python-packaging-user-guide.readthedocs.io/distributing/#classifiers>`_
    to make it easy for users to find your plugin.


Assertion Rewriting
-------------------

One of the main features of ``pytest`` is the use of plain assert
statements and the detailed introspection of expressions upon
assertion failures.  This is provided by "assertion rewriting" which
modifies the parsed AST before it gets compiled to bytecode.  This is
done via a :pep:`302` import hook which gets installed early on when
``pytest`` starts up and will perform this rewriting when modules get
imported.  However since we do not want to test different bytecode
then you will run in production this hook only rewrites test modules
themselves as well as any modules which are part of plugins.  Any
other imported module will not be rewritten and normal assertion
behaviour will happen.

If you have assertion helpers in other modules where you would need
assertion rewriting to be enabled you need to ask ``pytest``
explicitly to rewrite this module before it gets imported.

.. autofunction:: pytest.register_assert_rewrite

This is especially important when you write a pytest plugin which is
created using a package.  The import hook only treats ``conftest.py``
files and any modules which are listed in the ``pytest11`` entrypoint
as plugins.  As an example consider the following package::

   pytest_foo/__init__.py
   pytest_foo/plugin.py
   pytest_foo/helper.py

With the following typical ``setup.py`` extract:

.. code-block:: python

   setup(
      ...
      entry_points={'pytest11': ['foo = pytest_foo.plugin']},
      ...
   )

In this case only ``pytest_foo/plugin.py`` will be rewritten.  If the
helper module also contains assert statements which need to be
rewritten it needs to be marked as such, before it gets imported.
This is easiest by marking it for rewriting inside the
``__init__.py`` module, which will always be imported first when a
module inside a package is imported.  This way ``plugin.py`` can still
import ``helper.py`` normally.  The contents of
``pytest_foo/__init__.py`` will then need to look like this:

.. code-block:: python

   import pytest

   pytest.register_assert_rewrite('pytest_foo.helper')



Requiring/Loading plugins in a test module or conftest file
-----------------------------------------------------------

You can require plugins in a test module or a ``conftest.py`` file like this:

.. code-block:: python

    pytest_plugins = ["name1", "name2"]

When the test module or conftest plugin is loaded the specified plugins
will be loaded as well. Any module can be blessed as a plugin, including internal
application modules:

.. code-block:: python

    pytest_plugins = "myapp.testsupport.myplugin"

``pytest_plugins`` variables are processed recursively, so note that in the example above
if ``myapp.testsupport.myplugin`` also declares ``pytest_plugins``, the contents
of the variable will also be loaded as plugins, and so on.

This mechanism makes it easy to share fixtures within applications or even
external applications without the need to create external plugins using
the ``setuptools``'s entry point technique.

Plugins imported by ``pytest_plugins`` will also automatically be marked
for assertion rewriting (see :func:`pytest.register_assert_rewrite`).
However for this to have any effect the module must not be
imported already; if it was already imported at the time the
``pytest_plugins`` statement is processed, a warning will result and
assertions inside the plugin will not be rewritten.  To fix this you
can either call :func:`pytest.register_assert_rewrite` yourself before
the module is imported, or you can arrange the code to delay the
importing until after the plugin is registered.


Accessing another plugin by name
--------------------------------

If a plugin wants to collaborate with code from
another plugin it can obtain a reference through
the plugin manager like this:

.. sourcecode:: python

    plugin = config.pluginmanager.getplugin("name_of_plugin")

If you want to look at the names of existing plugins, use
the ``--trace-config`` option.

Testing plugins
---------------

pytest comes with a plugin named ``pytester`` that helps you write tests for
your plugin code. The plugin is disabled by default, so you will have to enable
it before you can use it.

You can do so by adding the following line to a ``conftest.py`` file in your
testing directory:

.. code-block:: python

    # content of conftest.py

    pytest_plugins = ["pytester"]

Alternatively you can invoke pytest with the ``-p pytester`` command line
option.

This will allow you to use the :py:class:`testdir <_pytest.pytester.Testdir>`
fixture for testing your plugin code.

Let's demonstrate what you can do with the plugin with an example. Imagine we
developed a plugin that provides a fixture ``hello`` which yields a function
and we can invoke this function with one optional parameter. It will return a
string value of ``Hello World!`` if we do not supply a value or ``Hello
{value}!`` if we do supply a string value.

.. code-block:: python

    # -*- coding: utf-8 -*-

    import pytest

    def pytest_addoption(parser):
        group = parser.getgroup('helloworld')
        group.addoption(
            '--name',
            action='store',
            dest='name',
            default='World',
            help='Default "name" for hello().'
        )

    @pytest.fixture
    def hello(request):
        name = request.config.getoption('name')

        def _hello(name=None):
            if not name:
                name = request.config.getoption('name')
            return "Hello {name}!".format(name=name)

        return _hello


Now the ``testdir`` fixture provides a convenient API for creating temporary
``conftest.py`` files and test files. It also allows us to run the tests and
return a result object, with which we can assert the tests' outcomes.

.. code-block:: python

    def test_hello(testdir):
        """Make sure that our plugin works."""

        # create a temporary conftest.py file
        testdir.makeconftest("""
            import pytest

            @pytest.fixture(params=[
                "Brianna",
                "Andreas",
                "Floris",
            ])
            def name(request):
                return request.param
        """)

        # create a temporary pytest test file
        testdir.makepyfile("""
            def test_hello_default(hello):
                assert hello() == "Hello World!"

            def test_hello_name(hello, name):
                assert hello(name) == "Hello {0}!".format(name)
        """)

        # run all tests with pytest
        result = testdir.runpytest()

        # check that all 4 tests passed
        result.assert_outcomes(passed=4)


For more information about the result object that ``runpytest()`` returns, and
the methods that it provides please check out the :py:class:`RunResult
<_pytest.pytester.RunResult>` documentation.


.. _`writinghooks`:

Writing hook functions
======================


.. _validation:

hook function validation and execution
--------------------------------------

pytest calls hook functions from registered plugins for any
given hook specification.  Let's look at a typical hook function
for the ``pytest_collection_modifyitems(session, config,
items)`` hook which pytest calls after collection of all test items is
completed.

When we implement a ``pytest_collection_modifyitems`` function in our plugin
pytest will during registration verify that you use argument
names which match the specification and bail out if not.

Let's look at a possible implementation:

.. code-block:: python

    def pytest_collection_modifyitems(config, items):
        # called after collection is completed
        # you can modify the ``items`` list

Here, ``pytest`` will pass in ``config`` (the pytest config object)
and ``items`` (the list of collected test items) but will not pass
in the ``session`` argument because we didn't list it in the function
signature.  This dynamic "pruning" of arguments allows ``pytest`` to
be "future-compatible": we can introduce new hook named parameters without
breaking the signatures of existing hook implementations.  It is one of
the reasons for the general long-lived compatibility of pytest plugins.

Note that hook functions other than ``pytest_runtest_*`` are not
allowed to raise exceptions.  Doing so will break the pytest run.



.. _firstresult:

firstresult: stop at first non-None result
-------------------------------------------

Most calls to ``pytest`` hooks result in a **list of results** which contains
all non-None results of the called hook functions.

Some hook specifications use the ``firstresult=True`` option so that the hook
call only executes until the first of N registered functions returns a
non-None result which is then taken as result of the overall hook call.
The remaining hook functions will not be called in this case.


hookwrapper: executing around other hooks
-------------------------------------------------

.. currentmodule:: _pytest.core

.. versionadded:: 2.7

pytest plugins can implement hook wrappers which wrap the execution
of other hook implementations.  A hook wrapper is a generator function
which yields exactly once. When pytest invokes hooks it first executes
hook wrappers and passes the same arguments as to the regular hooks.

At the yield point of the hook wrapper pytest will execute the next hook
implementations and return their result to the yield point in the form of
a :py:class:`Result <pluggy._Result>` instance which encapsulates a result or
exception info.  The yield point itself will thus typically not raise
exceptions (unless there are bugs).

Here is an example definition of a hook wrapper::

    import pytest

    @pytest.hookimpl(hookwrapper=True)
    def pytest_pyfunc_call(pyfuncitem):
        # do whatever you want before the next hook executes

        outcome = yield
        # outcome.excinfo may be None or a (cls, val, tb) tuple

        res = outcome.get_result()  # will raise if outcome was exception
        # postprocess result

Note that hook wrappers don't return results themselves, they merely
perform tracing or other side effects around the actual hook implementations.
If the result of the underlying hook is a mutable object, they may modify
that result but it's probably better to avoid it.


Hook function ordering / call example
-------------------------------------

For any given hook specification there may be more than one
implementation and we thus generally view ``hook`` execution as a
``1:N`` function call where ``N`` is the number of registered functions.
There are ways to influence if a hook implementation comes before or
after others, i.e.  the position in the ``N``-sized list of functions:

.. code-block:: python

    # Plugin 1
    @pytest.hookimpl(tryfirst=True)
    def pytest_collection_modifyitems(items):
        # will execute as early as possible

    # Plugin 2
    @pytest.hookimpl(trylast=True)
    def pytest_collection_modifyitems(items):
        # will execute as late as possible

    # Plugin 3
    @pytest.hookimpl(hookwrapper=True)
    def pytest_collection_modifyitems(items):
        # will execute even before the tryfirst one above!
        outcome = yield
        # will execute after all non-hookwrappers executed

Here is the order of execution:

1. Plugin3's pytest_collection_modifyitems called until the yield point
   because it is a hook wrapper.

2. Plugin1's pytest_collection_modifyitems is called because it is marked
   with ``tryfirst=True``.

3. Plugin2's pytest_collection_modifyitems is called because it is marked
   with ``trylast=True`` (but even without this mark it would come after
   Plugin1).

4. Plugin3's pytest_collection_modifyitems then executing the code after the yield
   point.  The yield receives a :py:class:`Result <pluggy._Result>` instance which encapsulates
   the result from calling the non-wrappers.  Wrappers shall not modify the result.

It's possible to use ``tryfirst`` and ``trylast`` also in conjunction with
``hookwrapper=True`` in which case it will influence the ordering of hookwrappers
among each other.


Declaring new hooks
------------------------

.. currentmodule:: _pytest.hookspec

Plugins and ``conftest.py`` files may declare new hooks that can then be
implemented by other plugins in order to alter behaviour or interact with
the new plugin:

.. autofunction:: pytest_addhooks

Hooks are usually declared as do-nothing functions that contain only
documentation describing when the hook will be called and what return values
are expected.

For an example, see `newhooks.py`_ from `xdist <https://github.com/pytest-dev/pytest-xdist>`_.

.. _`newhooks.py`: https://github.com/pytest-dev/pytest-xdist/blob/974bd566c599dc6a9ea291838c6f226197208b46/xdist/newhooks.py


Optionally using hooks from 3rd party plugins
---------------------------------------------

Using new hooks from plugins as explained above might be a little tricky
because of the standard :ref:`validation mechanism <validation>`:
if you depend on a plugin that is not installed, validation will fail and
the error message will not make much sense to your users.

One approach is to defer the hook implementation to a new plugin instead of
declaring the hook functions directly in your plugin module, for example::

    # contents of myplugin.py

    class DeferPlugin(object):
        """Simple plugin to defer pytest-xdist hook functions."""

        def pytest_testnodedown(self, node, error):
            """standard xdist hook function.
            """

    def pytest_configure(config):
        if config.pluginmanager.hasplugin('xdist'):
            config.pluginmanager.register(DeferPlugin())

This has the added benefit of allowing you to conditionally install hooks
depending on which plugins are installed.

.. _`well specified hooks`:

.. currentmodule:: _pytest.hookspec

pytest hook reference
=====================


Initialization, command line and configuration hooks
----------------------------------------------------

.. autofunction:: pytest_load_initial_conftests
.. autofunction:: pytest_cmdline_preparse
.. autofunction:: pytest_cmdline_parse
.. autofunction:: pytest_addoption
.. autofunction:: pytest_cmdline_main
.. autofunction:: pytest_configure
.. autofunction:: pytest_unconfigure

Generic "runtest" hooks
-----------------------

All runtest related hooks receive a :py:class:`pytest.Item <_pytest.main.Item>` object.

.. autofunction:: pytest_runtest_protocol
.. autofunction:: pytest_runtest_setup
.. autofunction:: pytest_runtest_call
.. autofunction:: pytest_runtest_teardown
.. autofunction:: pytest_runtest_makereport

For deeper understanding you may look at the default implementation of
these hooks in :py:mod:`_pytest.runner` and maybe also
in :py:mod:`_pytest.pdb` which interacts with :py:mod:`_pytest.capture`
and its input/output capturing in order to immediately drop
into interactive debugging when a test failure occurs.

The :py:mod:`_pytest.terminal` reported specifically uses
the reporting hook to print information about a test run.

Collection hooks
----------------

``pytest`` calls the following hooks for collecting files and directories:

.. autofunction:: pytest_ignore_collect
.. autofunction:: pytest_collect_directory
.. autofunction:: pytest_collect_file

For influencing the collection of objects in Python modules
you can use the following hook:

.. autofunction:: pytest_pycollect_makeitem
.. autofunction:: pytest_generate_tests
.. autofunction:: pytest_make_parametrize_id

After collection is complete, you can modify the order of
items, delete or otherwise amend the test items:

.. autofunction:: pytest_collection_modifyitems

Reporting hooks
---------------

Session related reporting hooks:

.. autofunction:: pytest_collectstart
.. autofunction:: pytest_itemcollected
.. autofunction:: pytest_collectreport
.. autofunction:: pytest_deselected
.. autofunction:: pytest_report_header
.. autofunction:: pytest_report_collectionfinish
.. autofunction:: pytest_report_teststatus
.. autofunction:: pytest_terminal_summary
.. autofunction:: pytest_fixture_setup
.. autofunction:: pytest_fixture_post_finalizer

And here is the central hook for reporting about
test execution:

.. autofunction:: pytest_runtest_logreport

You can also use this hook to customize assertion representation for some
types:

.. autofunction:: pytest_assertrepr_compare


Debugging/Interaction hooks
---------------------------

There are few hooks which can be used for special
reporting or interaction with exceptions:

.. autofunction:: pytest_internalerror
.. autofunction:: pytest_keyboard_interrupt
.. autofunction:: pytest_exception_interact
.. autofunction:: pytest_enter_pdb


Reference of objects involved in hooks
======================================

.. autoclass:: _pytest.config.Config()
    :members:

.. autoclass:: _pytest.config.Parser()
    :members:

.. autoclass:: _pytest.main.Node()
    :members:

.. autoclass:: _pytest.main.Collector()
    :members:
    :show-inheritance:

.. autoclass:: _pytest.main.Item()
    :members:
    :show-inheritance:

.. autoclass:: _pytest.python.Module()
    :members:
    :show-inheritance:

.. autoclass:: _pytest.python.Class()
    :members:
    :show-inheritance:

.. autoclass:: _pytest.python.Function()
    :members:
    :show-inheritance:

.. autoclass:: _pytest.fixtures.FixtureDef()
    :members:
    :show-inheritance:

.. autoclass:: _pytest.runner.CallInfo()
    :members:

.. autoclass:: _pytest.runner.TestReport()
    :members:
    :inherited-members:

.. autoclass:: pluggy._Result
    :members:

.. autofunction:: _pytest.config.get_plugin_manager()

.. autoclass:: _pytest.config.PytestPluginManager()
    :members:
    :undoc-members:
    :show-inheritance:

.. autoclass:: pluggy.PluginManager()
    :members:

.. currentmodule:: _pytest.pytester

.. autoclass:: Testdir()
    :members: runpytest,runpytest_subprocess,runpytest_inprocess,makeconftest,makepyfile

.. autoclass:: RunResult()
    :members:

.. autoclass:: LineMatcher()
    :members:
