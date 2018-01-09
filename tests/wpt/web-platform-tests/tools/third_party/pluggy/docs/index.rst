``pluggy``
==========

The ``pytest`` plugin system
****************************
``pluggy`` is the crystallized core of `plugin management and hook
calling`_ for `pytest`_.

In fact, ``pytest`` is itself composed as a set of ``pluggy`` plugins
which are invoked in sequence according to a well defined set of protocols.
Some `200+ plugins`_ use ``pluggy`` to extend and customize ``pytest``'s default behaviour.

In essence, ``pluggy`` enables function `hooking`_ so you can build "pluggable" systems.

How's it work?
--------------
A `plugin` is a `namespace`_ which defines hook functions.

``pluggy`` manages *plugins* by relying on:

- a hook *specification* - defines a call signature
- a set of hook *implementations* - aka `callbacks`_
- the hook *caller* - a call loop which collects results

where for each registered hook *specification*, a hook *call* will invoke up to ``N``
registered hook *implementations*.

``pluggy`` accomplishes all this by implementing a `request-response pattern`_ using *function*
subscriptions and can be thought of and used as a rudimentary busless `publish-subscribe`_
event system.

``pluggy``'s approach is meant to let a designer think carefuly about which objects are
explicitly needed by an extension writer. This is in contrast to subclass-based extension
systems which may expose unecessary state and behaviour or encourage `tight coupling`_
in overlying frameworks.


A first example
---------------

.. literalinclude:: examples/firstexample.py

Running this directly gets us::

    $ python docs/examples/firstexample.py

    inside Plugin_2.myhook()
    inside Plugin_1.myhook()
    [-1, 3]

For more details and advanced usage please read on.

.. _define:

Defining and Collecting Hooks
*****************************
A *plugin* is a namespace type (currently one of a ``class`` or module)
which defines a set of *hook* functions.

As mentioned in :ref:`manage`, all *plugins* which define *hooks*
are managed by an instance of a :py:class:`pluggy.PluginManager` which
defines the primary ``pluggy`` API.

In order for a ``PluginManager`` to detect functions in a namespace
intended to be *hooks*, they must be decorated using special ``pluggy`` *marks*.

.. _marking_hooks:

Marking hooks
-------------
The :py:class:`~pluggy.HookspecMarker` and :py:class:`~pluggy.HookimplMarker`
decorators are used to *mark* functions for detection by a ``PluginManager``:

.. code-block:: python

    from pluggy import HookspecMarker, HookimplMarker

    hookspec = HookspecMarker('project_name')
    hookimpl = HookimplMarker('project_name')


Each decorator type takes a single ``project_name`` string as its
lone argument the value of which is used to mark hooks for detection by
by a similarly configured ``PluginManager`` instance.

That is, a *mark* type called with ``project_name`` returns an object which
can be used to decorate functions which will then be detected by a
``PluginManager`` which was instantiated with the the same ``project_name``
value.

Furthermore, each *hookimpl* or *hookspec* decorator can configure the
underlying call-time behavior of each *hook* object by providing special
*options* passed as keyword arguments.


.. note::
    The following sections correspond to similar documentation in
    ``pytest`` for `Writing hook functions`_ and can be used
    as a supplementary resource.

.. _impls:

Implementations
---------------
A hook *implementation* (*hookimpl*) is just a (callback) function
which has been appropriately marked.

*hookimpls* are loaded from a plugin using the
:py:meth:`~pluggy.PluginManager.register()` method:

.. code-block:: python

    import sys
    from pluggy import PluginManager, HookimplMarker

    hookimpl = HookimplMarker('myproject')

    @hookimpl
    def setup_project(config, args):
        """This hook is used to process the initial config
        and possibly input arguments.
        """
        if args:
            config.process_args(args)

        return config

    pm = PluginManager('myproject')

    # load all hookimpls from the local module's namespace
    plugin_name = pm.register(sys.modules[__name__])

.. _optionalhook:

Optional validation
^^^^^^^^^^^^^^^^^^^
Normally each *hookimpl* should be validated a against a corresponding
hook :ref:`specification <specs>`. If you want to make an exception
then the *hookimpl* should be marked with the ``"optionalhook"`` option:

.. code-block:: python

    @hookimpl(optionalhook=True)
    def setup_project(config, args):
        """This hook is used to process the initial config
        and possibly input arguments.
        """
        if args:
            config.process_args(args)

        return config

Call time order
^^^^^^^^^^^^^^^
By default hooks are :ref:`called <calling>` in LIFO registered order, however,
a *hookimpl* can influence its call-time invocation position using special
attributes. If marked with a ``"tryfirst"`` or ``"trylast"`` option it
will be executed *first* or *last* respectively in the hook call loop:

.. code-block:: python

    import sys
    from pluggy import PluginManager, HookimplMarker

    hookimpl = HookimplMarker('myproject')

    @hookimpl(trylast=True)
    def setup_project(config, args):
        """Default implementation.
        """
        if args:
            config.process_args(args)

        return config


    class SomeOtherPlugin(object):
        """Some other plugin defining the same hook.
        """
        @hookimpl(tryfirst=True)
        def setup_project(config, args):
            """Report what args were passed before calling
            downstream hooks.
            """
            if args:
                print("Got args: {}".format(args))

            return config

    pm = PluginManager('myproject')

    # load from the local module's namespace
    pm.register(sys.modules[__name__])
    # load a plugin defined on a class
    pm.register(SomePlugin())

For another example see the `hook function ordering`_ section of the
``pytest`` docs.

.. note::
    ``tryfirst`` and ``trylast`` hooks are still invoked in LIFO order within
    each category.

Wrappers
^^^^^^^^
A *hookimpl* can be marked with a ``"hookwrapper"`` option which indicates that
the function will be called to *wrap* (or surround) all other normal *hookimpl*
calls. A *hookwrapper* can thus execute some code ahead and after the execution
of all corresponding non-wrappper *hookimpls*.

Much in the same way as a `@contextlib.contextmanager`_, *hookwrappers* must
be implemented as generator function with a single ``yield`` in its body:


.. code-block:: python

    @hookimpl(hookwrapper=True)
    def setup_project(config, args):
        """Wrap calls to ``setup_project()`` implementations which
        should return json encoded config options.
        """
        if config.debug:
            print("Pre-hook config is {}".format(
                config.tojson()))

        # get initial default config
        defaults = config.tojson()

        # all corresponding hookimpls are invoked here
        outcome = yield

        for item in outcome.get_result():
            print("JSON config override is {}".format(item))

        if config.debug:
            print("Post-hook config is {}".format(
                config.tojson()))

        if config.use_defaults:
            outcome.force_result(defaults)

The generator is `sent`_ a :py:class:`pluggy._Result` object which can
be assigned in the ``yield`` expression and used to override or inspect
the final result(s) returned back to the caller using the
:py:meth:`~pluggy._Result.force_result` or
:py:meth:`~pluggy._Result.get_result` methods.

.. note::
    Hook wrappers can **not** return results (as per generator function
    semantics); they can only modify them using the ``_Result`` API.

Also see the `hookwrapper`_ section in the ``pytest`` docs.

.. _specs:

Specifications
--------------
A hook *specification* (*hookspec*) is a definition used to validate each
*hookimpl* ensuring that an extension writer has correctly defined their
callback function *implementation* .

*hookspecs* are defined using similarly marked functions however only the
function *signature* (its name and names of all its arguments) is analyzed
and stored. As such, often you will see a *hookspec* defined with only
a docstring in its body.

*hookspecs* are loaded using the
:py:meth:`~pluggy.PluginManager.add_hookspecs()` method and normally
should be added before registering corresponding *hookimpls*:

.. code-block:: python

    import sys
    from pluggy import PluginManager, HookspecMarker

    hookspec = HookspecMarker('myproject')

    @hookspec
    def setup_project(config, args):
        """This hook is used to process the inital config and input
        arguments.
        """

    pm = PluginManager('myproject')

    # load from the local module's namespace
    pm.add_hookspecs(sys.modules[__name__])


Registering a *hookimpl* which does not meet the constraints of its
corresponding *hookspec* will result in an error.

A *hookspec* can also be added **after** some *hookimpls* have been
registered however this is not normally recommended as it results in
delayed hook validation.

.. note::
    The term *hookspec* can sometimes refer to the plugin-namespace
    which defines ``hookspec`` decorated functions as in the case of
    ``pytest``'s `hookspec module`_

Enforcing spec validation
^^^^^^^^^^^^^^^^^^^^^^^^^
By default there is no strict requirement that each *hookimpl* has
a corresponding *hookspec*. However, if you'd like you enforce this
behavior you can run a check with the
:py:meth:`~pluggy.PluginManager.check_pending()` method. If you'd like
to enforce requisite *hookspecs* but with certain exceptions for some hooks
then make sure to mark those hooks as :ref:`optional <optionalhook>`.

Opt-in arguments
^^^^^^^^^^^^^^^^
To allow for *hookspecs* to evolve over the lifetime of a project,
*hookimpls* can accept **less** arguments then defined in the spec.
This allows for extending hook arguments (and thus semantics) without
breaking existing *hookimpls*.

In other words this is ok:

.. code-block:: python

    @hookspec
    def myhook(config, args):
        pass

    @hookimpl
    def myhook(args):
        print(args)


whereas this is not:

.. code-block:: python

    @hookspec
    def myhook(config, args):
        pass

    @hookimpl
    def myhook(config, args, extra_arg):
        print(args)

.. _firstresult:

First result only
^^^^^^^^^^^^^^^^^
A *hookspec* can be marked such that when the *hook* is called the call loop
will only invoke up to the first *hookimpl* which returns a result other
then ``None``.

.. code-block:: python

    @hookspec(firstresult=True)
    def myhook(config, args):
        pass

This can be useful for optimizing a call loop for which you are only
interested in a single core *hookimpl*. An example is the
`pytest_cmdline_main`_ central routine of ``pytest``.

Also see the `first result`_ section in the ``pytest`` docs.

.. _historic:

Historic hooks
^^^^^^^^^^^^^^
You can mark a *hookspec* as being *historic* meaning that the hook
can be called with :py:meth:`~pluggy.PluginManager.call_historic()` **before**
having been registered:

.. code-block:: python

    @hookspec(historic=True)
    def myhook(config, args):
        pass

The implication is that late registered *hookimpls* will be called back
immediately at register time and **can not** return a result to the caller.**

This turns out to be particularly useful when dealing with lazy or
dynamically loaded plugins.

For more info see :ref:`call_historic`.


.. links
.. _@contextlib.contextmanager:
    https://docs.python.org/3.6/library/contextlib.html#contextlib.contextmanager
.. _pytest_cmdline_main:
    https://github.com/pytest-dev/pytest/blob/master/_pytest/hookspec.py#L80
.. _hookspec module:
    https://github.com/pytest-dev/pytest/blob/master/_pytest/hookspec.py
.. _Writing hook functions:
    http://doc.pytest.org/en/latest/writing_plugins.html#writing-hook-functions
.. _hookwrapper:
    http://doc.pytest.org/en/latest/writing_plugins.html#hookwrapper-executing-around-other-hooks
.. _hook function ordering:
    http://doc.pytest.org/en/latest/writing_plugins.html#hook-function-ordering-call-example
.. _first result:
    http://doc.pytest.org/en/latest/writing_plugins.html#firstresult-stop-at-first-non-none-result
.. _sent:
    https://docs.python.org/3/reference/expressions.html#generator.send

.. _manage:

The Plugin Registry
*******************
``pluggy`` manages plugins using instances of the
:py:class:`pluggy.PluginManager`.

A ``PluginManager`` is instantiated with a single
``str`` argument, the ``project_name``:

.. code-block:: python

    import pluggy
    pm = pluggy.PluginManager('my_project_name')


The ``project_name`` value is used when a ``PluginManager`` scans for *hook*
functions :ref:`defined on a plugin <define>`.
This allows for multiple
plugin managers from multiple projects to define hooks alongside each other.


Registration
------------
Each ``PluginManager`` maintains a *plugin* registry where each *plugin*
contains a set of *hookimpl* definitions. Loading *hookimpl* and *hookspec*
definitions to populate the registry is described in detail in the section on
:ref:`define`.

In summary, you pass a plugin namespace object to the
:py:meth:`~pluggy.PluginManager.register()` and
:py:meth:`~pluggy.PluginManager.add_hookspec()` methods to collect
hook *implementations* and *specfications* from *plugin* namespaces respectively.

You can unregister any *plugin*'s hooks using
:py:meth:`~pluggy.PluginManager.unregister()` and check if a plugin is
registered by passing its name to the
:py:meth:`~pluggy.PluginManager.is_registered()` method.

Loading ``setuptools`` entry points
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
You can automatically load plugins registered through `setuptools entry points`_
with the :py:meth:`~pluggy.PluginManager.load_setuptools_entrypoints()`
method.

An example use of this is the `pytest entry point`_.


Blocking
--------
You can block any plugin from being registered using
:py:meth:`~pluggy.PluginManager.set_blocked()` and check if a given
*plugin* is blocked by name using :py:meth:`~pluggy.PluginManager.is_blocked()`.


Inspection
----------
You can use a variety of methods to inspect the both the registry
and particular plugins in it:

- :py:meth:`~pluggy.PluginManager.list_name_plugin()` -
  return a list of name-plugin pairs
- :py:meth:`~pluggy.PluginManager.get_plugins()` - retrieve all plugins
- :py:meth:`~pluggy.PluginManager.get_canonical_name()`- get a *plugin*'s
  canonical name (the name it was registered with)
- :py:meth:`~pluggy.PluginManager.get_plugin()` - retrieve a plugin by its
  canonical name


Parsing mark options
^^^^^^^^^^^^^^^^^^^^
You can retrieve the *options* applied to a particular
*hookspec* or *hookimpl* as per :ref:`marking_hooks` using the
:py:meth:`~pluggy.PluginManager.parse_hookspec_opts()` and
:py:meth:`~pluggy.PluginManager.parse_hookimpl_opts()` respectively.

.. links
.. _setuptools entry points:
    http://setuptools.readthedocs.io/en/latest/setuptools.html#dynamic-discovery-of-services-and-plugins
.. _pytest entry point:
    http://doc.pytest.org/en/latest/writing_plugins.html#setuptools-entry-points


.. _calling:

Calling Hooks
*************
The core functionality of ``pluggy`` enables an extension provider
to override function calls made at certain points throughout a program.

A particular *hook* is invoked by calling an instance of
a :py:class:`pluggy._HookCaller` which in turn *loops* through the
``1:N`` registered *hookimpls* and calls them in sequence.

Every :py:class:`pluggy.PluginManager` has a ``hook`` attribute
which is an instance of this :py:class:`pluggy._HookRelay`.
The ``_HookRelay`` itself contains references (by hook name) to each
registered *hookimpl*'s ``_HookCaller`` instance.

More practically you call a *hook* like so:

.. code-block:: python

    import sys
    import pluggy
    import mypluginspec
    import myplugin
    from configuration import config

    pm = pluggy.PluginManager("myproject")
    pm.add_hookspecs(mypluginspec)
    pm.register(myplugin)

    # we invoke the _HookCaller and thus all underlying hookimpls
    result_list = pm.hook.myhook(config=config, args=sys.argv)

Note that you **must** call hooks using keyword `arguments`_ syntax!

Hook implementations are called in LIFO registered order: *the last
registered plugin's hooks are called first*. As an example, the below
assertion should not error:

.. code-block:: python

    from pluggy import PluginManager, HookimplMarker

    hookimpl = HookimplMarker('myproject')

    class Plugin1(object):
        def myhook(self, args):
            """Default implementation.
            """
            return 1

    class Plugin2(object):
        def myhook(self, args):
            """Default implementation.
            """
            return 2

    class Plugin3(object):
        def myhook(self, args):
            """Default implementation.
            """
            return 3

    pm = PluginManager('myproject')
    pm.register(Plugin1())
    pm.register(Plugin2())
    pm.register(Plugin3())

    assert pm.hook.myhook(args=()) == [3, 2, 1]

Collecting results
------------------
By default calling a hook results in all underlying :ref:`hookimpls
<impls>` functions to be invoked in sequence via a loop. Any function
which returns a value other then a ``None`` result will have that result
appended to a :py:class:`list` which is returned by the call.

The only exception to this behaviour is if the hook has been marked to return
its :ref:`firstresult` in which case only the first single value (which is not
``None``) will be returned.

.. _call_historic:

Historic calls
--------------
A *historic call* allows for all newly registered functions to receive all hook
calls that happened before their registration. The implication is that this is
only useful if you expect that some *hookimpls* may be registered **after** the
hook is initially invoked.

Historic hooks must be :ref:`specially marked <historic>` and called
using the :py:meth:`pluggy._HookCaller.call_historic()` method:

.. code-block:: python

    # call with history; no results returned
    pm.hook.myhook.call_historic(config=config, args=sys.argv)

    # ... more of our program ...

    # late loading of some plugin
    import mylateplugin

    # historic call back is done here
    pm.register(mylateplugin)

Note that if you ``call_historic()`` the ``_HookCaller`` (and thus your
calling code) can not receive results back from the underlying *hookimpl*
functions.

Calling with extras
-------------------
You can call a hook with temporarily participating *implementation* functions
(that aren't in the registry) using the
:py:meth:`pluggy._HookCaller.call_extra()` method.


Calling with a subset of registered plugins
-------------------------------------------
You can make a call using a subset of plugins by asking the
``PluginManager`` first for a ``_HookCaller`` with those plugins removed
using the :py:meth:`pluggy.PluginManager.subset_hook_caller()` method.

You then can use that ``_HookCaller`` to make normal, ``call_historic()``,
or ``call_extra()`` calls as necessary.


.. links
.. _arguments:
    https://docs.python.org/3/glossary.html#term-argument


Built-in tracing
****************
``pluggy`` comes with some batteries included hook tracing for your
debugging needs.


Call tracing
------------
To enable tracing use the
:py:meth:`pluggy.PluginManager.enable_tracing()` method which returns an
undo function to disable the behaviour.

.. code-block:: python

    pm = PluginManager('myproject')
    # magic line to set a writer function
    pm.trace.root.setwriter(print)
    undo = pm.enable_tracing()


Call monitoring
---------------
Instead of using the built-in tracing mechanism you can also add your
own ``before`` and ``after`` monitoring functions using
:py:class:`pluggy.PluginManager.add_hookcall_monitoring()`.

The expected signature and default implementations for these functions is:

.. code-block:: python

    def before(hook_name, methods, kwargs):
        pass

    def after(outcome, hook_name, methods, kwargs):
        pass

Public API
**********
Please see the :doc:`api_reference`.

Development
***********
Great care must taken when hacking on ``pluggy`` since multiple mature
projects rely on it. Our Github integrated CI process runs the full
`tox test suite`_ on each commit so be sure your changes can run on
all required `Python interpreters`_ and ``pytest`` versions.

Release Policy
**************
Pluggy uses `Semantic Versioning`_. Breaking changes are only foreseen for
Major releases (incremented X in "X.Y.Z").  If you want to use ``pluggy``
in your project you should thus use a dependency restriction like
``"pluggy>=0.1.0,<1.0"`` to avoid surprises.


.. hyperlinks
.. _pytest:
    http://pytest.org
.. _request-response pattern:
    https://en.wikipedia.org/wiki/Request%E2%80%93response
.. _publish-subscribe:
    https://en.wikipedia.org/wiki/Publish%E2%80%93subscribe_pattern
.. _hooking:
    https://en.wikipedia.org/wiki/Hooking
.. _plugin management and hook calling:
    http://doc.pytest.org/en/latest/writing_plugins.html
.. _namespace:
    https://docs.python.org/3.6/tutorial/classes.html#python-scopes-and-namespaces
.. _callbacks:
    https://en.wikipedia.org/wiki/Callback_(computer_programming)
.. _tox test suite:
    https://github.com/pytest-dev/pluggy/blob/master/tox.ini
.. _Semantic Versioning:
    http://semver.org/
.. _tight coupling:
    https://en.wikipedia.org/wiki/Coupling_%28computer_programming%29#Types_of_coupling
.. _Python interpreters:
    https://github.com/pytest-dev/pluggy/blob/master/tox.ini#L2
.. _200+ plugins:
    http://plugincompat.herokuapp.com/


.. Indices and tables
.. ==================
.. * :ref:`genindex`
.. * :ref:`modindex`
.. * :ref:`search`
