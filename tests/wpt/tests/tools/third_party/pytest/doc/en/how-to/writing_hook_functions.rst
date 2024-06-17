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
        ...

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

.. _`hookwrapper`:

hook wrappers: executing around other hooks
-------------------------------------------------

pytest plugins can implement hook wrappers which wrap the execution
of other hook implementations.  A hook wrapper is a generator function
which yields exactly once. When pytest invokes hooks it first executes
hook wrappers and passes the same arguments as to the regular hooks.

At the yield point of the hook wrapper pytest will execute the next hook
implementations and return their result to the yield point, or will
propagate an exception if they raised.

Here is an example definition of a hook wrapper:

.. code-block:: python

    import pytest


    @pytest.hookimpl(wrapper=True)
    def pytest_pyfunc_call(pyfuncitem):
        do_something_before_next_hook_executes()

        # If the outcome is an exception, will raise the exception.
        res = yield

        new_res = post_process_result(res)

        # Override the return value to the plugin system.
        return new_res

The hook wrapper needs to return a result for the hook, or raise an exception.

In many cases, the wrapper only needs to perform tracing or other side effects
around the actual hook implementations, in which case it can return the result
value of the ``yield``. The simplest (though useless) hook wrapper is
``return (yield)``.

In other cases, the wrapper wants the adjust or adapt the result, in which case
it can return a new value. If the result of the underlying hook is a mutable
object, the wrapper may modify that result, but it's probably better to avoid it.

If the hook implementation failed with an exception, the wrapper can handle that
exception using a ``try-catch-finally`` around the ``yield``, by propagating it,
suppressing it, or raising a different exception entirely.

For more information, consult the
:ref:`pluggy documentation about hook wrappers <pluggy:hookwrappers>`.

.. _plugin-hookorder:

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
        ...


    # Plugin 2
    @pytest.hookimpl(trylast=True)
    def pytest_collection_modifyitems(items):
        # will execute as late as possible
        ...


    # Plugin 3
    @pytest.hookimpl(wrapper=True)
    def pytest_collection_modifyitems(items):
        # will execute even before the tryfirst one above!
        try:
            return (yield)
        finally:
            # will execute after all non-wrappers executed
            ...

Here is the order of execution:

1. Plugin3's pytest_collection_modifyitems called until the yield point
   because it is a hook wrapper.

2. Plugin1's pytest_collection_modifyitems is called because it is marked
   with ``tryfirst=True``.

3. Plugin2's pytest_collection_modifyitems is called because it is marked
   with ``trylast=True`` (but even without this mark it would come after
   Plugin1).

4. Plugin3's pytest_collection_modifyitems then executing the code after the yield
   point.  The yield receives the result from calling the non-wrappers, or raises
   an exception if the non-wrappers raised.

It's possible to use ``tryfirst`` and ``trylast`` also on hook wrappers
in which case it will influence the ordering of hook wrappers among each other.

.. _`declaringhooks`:

Declaring new hooks
------------------------

.. note::

    This is a quick overview on how to add new hooks and how they work in general, but a more complete
    overview can be found in `the pluggy documentation <https://pluggy.readthedocs.io/en/latest/>`__.

Plugins and ``conftest.py`` files may declare new hooks that can then be
implemented by other plugins in order to alter behaviour or interact with
the new plugin:

.. autofunction:: _pytest.hookspec.pytest_addhooks
    :noindex:

Hooks are usually declared as do-nothing functions that contain only
documentation describing when the hook will be called and what return values
are expected. The names of the functions must start with `pytest_` otherwise pytest won't recognize them.

Here's an example. Let's assume this code is in the ``sample_hook.py`` module.

.. code-block:: python

    def pytest_my_hook(config):
        """
        Receives the pytest config and does things with it
        """

To register the hooks with pytest they need to be structured in their own module or class. This
class or module can then be passed to the ``pluginmanager`` using the ``pytest_addhooks`` function
(which itself is a hook exposed by pytest).

.. code-block:: python

    def pytest_addhooks(pluginmanager):
        """This example assumes the hooks are grouped in the 'sample_hook' module."""
        from my_app.tests import sample_hook

        pluginmanager.add_hookspecs(sample_hook)

For a real world example, see `newhooks.py`_ from `xdist <https://github.com/pytest-dev/pytest-xdist>`_.

.. _`newhooks.py`: https://github.com/pytest-dev/pytest-xdist/blob/974bd566c599dc6a9ea291838c6f226197208b46/xdist/newhooks.py

Hooks may be called both from fixtures or from other hooks. In both cases, hooks are called
through the ``hook`` object, available in the ``config`` object. Most hooks receive a
``config`` object directly, while fixtures may use the ``pytestconfig`` fixture which provides the same object.

.. code-block:: python

    @pytest.fixture()
    def my_fixture(pytestconfig):
        # call the hook called "pytest_my_hook"
        # 'result' will be a list of return values from all registered functions.
        result = pytestconfig.hook.pytest_my_hook(config=pytestconfig)

.. note::
    Hooks receive parameters using only keyword arguments.

Now your hook is ready to be used. To register a function at the hook, other plugins or users must
now simply define the function ``pytest_my_hook`` with the correct signature in their ``conftest.py``.

Example:

.. code-block:: python

    def pytest_my_hook(config):
        """
        Print all active hooks to the screen.
        """
        print(config.hook)


.. _`addoptionhooks`:


Using hooks in pytest_addoption
-------------------------------

Occasionally, it is necessary to change the way in which command line options
are defined by one plugin based on hooks in another plugin. For example,
a plugin may expose a command line option for which another plugin needs
to define the default value. The pluginmanager can be used to install and
use hooks to accomplish this. The plugin would define and add the hooks
and use pytest_addoption as follows:

.. code-block:: python

   # contents of hooks.py


   # Use firstresult=True because we only want one plugin to define this
   # default value
   @hookspec(firstresult=True)
   def pytest_config_file_default_value():
       """Return the default value for the config file command line option."""


   # contents of myplugin.py


   def pytest_addhooks(pluginmanager):
       """This example assumes the hooks are grouped in the 'hooks' module."""
       from . import hooks

       pluginmanager.add_hookspecs(hooks)


   def pytest_addoption(parser, pluginmanager):
       default_value = pluginmanager.hook.pytest_config_file_default_value()
       parser.addoption(
           "--config-file",
           help="Config file to use, defaults to %(default)s",
           default=default_value,
       )

The conftest.py that is using myplugin would simply define the hook as follows:

.. code-block:: python

    def pytest_config_file_default_value():
        return "config.yaml"


Optionally using hooks from 3rd party plugins
---------------------------------------------

Using new hooks from plugins as explained above might be a little tricky
because of the standard :ref:`validation mechanism <validation>`:
if you depend on a plugin that is not installed, validation will fail and
the error message will not make much sense to your users.

One approach is to defer the hook implementation to a new plugin instead of
declaring the hook functions directly in your plugin module, for example:

.. code-block:: python

    # contents of myplugin.py


    class DeferPlugin:
        """Simple plugin to defer pytest-xdist hook functions."""

        def pytest_testnodedown(self, node, error):
            """standard xdist hook function."""


    def pytest_configure(config):
        if config.pluginmanager.hasplugin("xdist"):
            config.pluginmanager.register(DeferPlugin())

This has the added benefit of allowing you to conditionally install hooks
depending on which plugins are installed.

.. _plugin-stash:

Storing data on items across hook functions
-------------------------------------------

Plugins often need to store data on :class:`~pytest.Item`\s in one hook
implementation, and access it in another. One common solution is to just
assign some private attribute directly on the item, but type-checkers like
mypy frown upon this, and it may also cause conflicts with other plugins.
So pytest offers a better way to do this, :attr:`item.stash <_pytest.nodes.Node.stash>`.

To use the "stash" in your plugins, first create "stash keys" somewhere at the
top level of your plugin:

.. code-block:: python

    been_there_key = pytest.StashKey[bool]()
    done_that_key = pytest.StashKey[str]()

then use the keys to stash your data at some point:

.. code-block:: python

    def pytest_runtest_setup(item: pytest.Item) -> None:
        item.stash[been_there_key] = True
        item.stash[done_that_key] = "no"

and retrieve them at another point:

.. code-block:: python

    def pytest_runtest_teardown(item: pytest.Item) -> None:
        if not item.stash[been_there_key]:
            print("Oh?")
        item.stash[done_that_key] = "yes!"

Stashes are available on all node types (like :class:`~pytest.Class`,
:class:`~pytest.Session`) and also on :class:`~pytest.Config`, if needed.
