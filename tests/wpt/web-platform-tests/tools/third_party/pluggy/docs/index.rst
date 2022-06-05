``pluggy``
==========
**The pytest plugin system**

What is it?
***********
``pluggy`` is the crystallized core of :ref:`plugin management and hook
calling <pytest:writing-plugins>` for :std:doc:`pytest <pytest:index>`.
It enables `500+ plugins`_ to extend and customize ``pytest``'s default
behaviour. Even ``pytest`` itself is composed as a set of ``pluggy`` plugins
which are invoked in sequence according to a well defined set of protocols.

It gives users the ability to extend or modify the behaviour of a
``host program`` by installing a ``plugin`` for that program.
The plugin code will run as part of normal program execution, changing or
enhancing certain aspects of it.

In essence, ``pluggy`` enables function `hooking`_ so you can build
"pluggable" systems.

Why is it useful?
*****************
There are some established mechanisms for modifying the behavior of other
programs/libraries in Python like
`method overriding <https://en.wikipedia.org/wiki/Method_overriding>`_
(e.g. Jinja2) or
`monkey patching <https://en.wikipedia.org/wiki/Monkey_patch>`_ (e.g. gevent
or for :std:doc:`writing tests <pytest:how-to/monkeypatch>`).
These strategies become problematic though when several parties want to
participate in the modification of the same program. Therefore ``pluggy``
does not rely on these mechanisms to enable a more structured approach and
avoid unnecessary exposure of state and behaviour. This leads to a more
`loosely coupled <https://en.wikipedia.org/wiki/Loose_coupling>`_ relationship
between ``host`` and ``plugins``.

The ``pluggy`` approach puts the burden on the designer of the
``host program`` to think carefully about which objects are really
needed in a hook implementation. This gives ``plugin`` creators a clear
framework for how to extend the ``host`` via a well defined set of functions
and objects to work with.

How does it work?
*****************
Let us start with a short overview of what is involved:

* ``host`` or ``host program``: the program offering extensibility
  by specifying ``hook functions`` and invoking their implementation(s) as
  part of program execution
* ``plugin``: the program implementing (a subset of) the specified hooks and
  participating in program execution when the implementations are invoked
  by the ``host``
* ``pluggy``: connects ``host`` and ``plugins`` by using ...

    - the hook :ref:`specifications <specs>` defining call signatures
      provided by the ``host`` (a.k.a ``hookspecs`` - see :ref:`marking_hooks`)
    - the hook :ref:`implementations <impls>` provided by registered
      ``plugins`` (a.k.a ``hookimpl`` - see `callbacks`_)
    - the hook :ref:`caller <calling>` - a call loop triggered at appropriate
      program positions in the ``host`` invoking the implementations and
      collecting the results

    ... where for each registered hook *specification*, a hook *call* will
    invoke up to ``N`` registered hook *implementations*.
* ``user``: the person who installed the ``host program`` and wants to
  extend its functionality with ``plugins``. In the simplest case they install
  the ``plugin`` in the same environment as the ``host`` and the magic will
  happen when the ``host program`` is run the next time. Depending on
  the ``plugin``, there might be other things they need to do. For example,
  they might have to call the host with an additional commandline parameter
  to the host that the ``plugin`` added.

A toy example
-------------
Let us demonstrate the core functionality in one module and show how you can
start experimenting with pluggy functionality.

.. literalinclude:: examples/toy-example.py

Running this directly gets us::

    $ python docs/examples/toy-example.py

    inside Plugin_2.myhook()
    inside Plugin_1.myhook()
    [-1, 3]

A complete example
------------------
Now let us demonstrate how this plays together in a vaguely real world scenario.

Let's assume our ``host program`` is called **eggsample** where some eggs will
be prepared and served with a tray containing condiments. As everybody knows:
the more cooks are involved the better the food, so let us make the process
pluggable and write a plugin that improves the meal with some spam and replaces
the steak sauce (nobody likes that anyway) with spam sauce (it's a thing - trust me).

.. note::

    **naming markers**: ``HookSpecMarker`` and ``HookImplMarker`` must be
    initialized with the name of the ``host`` project (the ``name``
    parameter in ``setup()``) - so **eggsample** in our case.

    **naming plugin projects**: they should be named in the form of
    ``<host>-<plugin>`` (e.g. ``pytest-xdist``), therefore we call our
    plugin *eggsample-spam*.

The host
^^^^^^^^
``eggsample/eggsample/__init__.py``

.. literalinclude:: examples/eggsample/eggsample/__init__.py

``eggsample/eggsample/hookspecs.py``

.. literalinclude:: examples/eggsample/eggsample/hookspecs.py

``eggsample/eggsample/lib.py``

.. literalinclude:: examples/eggsample/eggsample/lib.py

``eggsample/eggsample/host.py``

.. literalinclude:: examples/eggsample/eggsample/host.py

``eggsample/setup.py``

.. literalinclude:: examples/eggsample/setup.py

Let's get cooking - we install the host and see what a program run looks like::

    $ pip install --editable pluggy/docs/examples/eggsample
    $ eggsample

    Your food. Enjoy some egg, egg, salt, egg, egg, pepper, egg
    Some condiments? We have pickled walnuts, steak sauce, mushy peas, mint sauce

The plugin
^^^^^^^^^^
``eggsample-spam/eggsample_spam.py``

.. literalinclude:: examples/eggsample-spam/eggsample_spam.py

``eggsample-spam/setup.py``

.. literalinclude:: examples/eggsample-spam/setup.py

Let's get cooking with more cooks - we install the plugin and and see what
we get::

    $ pip install --editable pluggy/docs/examples/eggsample-spam
    $ eggsample

    Your food. Enjoy some egg, lovely spam, salt, egg, egg, egg, wonderous spam, egg, pepper
    Some condiments? We have pickled walnuts, mushy peas, mint sauce, spam sauce
    Now this is what I call a condiments tray!

More real world examples
------------------------
To see how ``pluggy`` is used in the real world, have a look at these projects
documentation and source code:

* :ref:`pytest <pytest:writing-plugins>`
* :std:doc:`tox <tox:plugins>`
* :std:doc:`devpi <devpi:devguide/index>`

For more details and advanced usage please read on.

.. _define:

Define and collect hooks
************************
A *plugin* is a :ref:`namespace <python:tut-scopes>` type (currently one of a
``class`` or module) which defines a set of *hook* functions.

As mentioned in :ref:`manage`, all *plugins* which specify *hooks*
are managed by an instance of a :py:class:`pluggy.PluginManager` which
defines the primary ``pluggy`` API.

In order for a :py:class:`~pluggy.PluginManager` to detect functions in a namespace
intended to be *hooks*, they must be decorated using special ``pluggy`` *marks*.

.. _marking_hooks:

Marking hooks
-------------
The :py:class:`~pluggy.HookspecMarker` and :py:class:`~pluggy.HookimplMarker`
decorators are used to *mark* functions for detection by a
:py:class:`~pluggy.PluginManager`:

.. code-block:: python

    from pluggy import HookspecMarker, HookimplMarker

    hookspec = HookspecMarker("project_name")
    hookimpl = HookimplMarker("project_name")


Each decorator type takes a single ``project_name`` string as its
lone argument the value of which is used to mark hooks for detection by
a similarly configured :py:class:`~pluggy.PluginManager` instance.

That is, a *mark* type called with ``project_name`` returns an object which
can be used to decorate functions which will then be detected by a
:py:class:`~pluggy.PluginManager` which was instantiated with the same
``project_name`` value.

Furthermore, each *hookimpl* or *hookspec* decorator can configure the
underlying call-time behavior of each *hook* object by providing special
*options* passed as keyword arguments.


.. note::
    The following sections correspond to similar documentation in
    ``pytest`` for :ref:`pytest:writinghooks` and can be used as
    a supplementary resource.

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

    hookimpl = HookimplMarker("myproject")


    @hookimpl
    def setup_project(config, args):
        """This hook is used to process the initial config
        and possibly input arguments.
        """
        if args:
            config.process_args(args)

        return config


    pm = PluginManager("myproject")

    # load all hookimpls from the local module's namespace
    plugin_name = pm.register(sys.modules[__name__])

.. _optionalhook:

Optional validation
^^^^^^^^^^^^^^^^^^^
Normally each *hookimpl* should be validated against a corresponding
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

.. _specname:

Hookspec name matching
^^^^^^^^^^^^^^^^^^^^^^

During plugin :ref:`registration <registration>`, pluggy attempts to match each
hook implementation declared by the *plugin* to a hook
:ref:`specification <specs>` in the *host* program with the **same name** as
the function being decorated by ``@hookimpl`` (e.g. ``setup_project`` in the
example above).  Note: there is *no* strict requirement that each *hookimpl*
has a corresponding *hookspec* (see
:ref:`enforcing spec validation <enforcing>`).

*new in version 0.13.2:*

To override the default behavior, a *hookimpl* may also be matched to a
*hookspec* in the *host* program with a non-matching function name by using
the ``specname`` option.  Continuing the example above, the *hookimpl* function
does not need to be named ``setup_project``, but if the argument
``specname="setup_project"`` is provided to the ``hookimpl`` decorator, it will
be matched and checked against the ``setup_project`` hookspec:

.. code-block:: python

    @hookimpl(specname="setup_project")
    def any_plugin_function(config, args):
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

    hookimpl = HookimplMarker("myproject")


    @hookimpl(trylast=True)
    def setup_project(config, args):
        """Default implementation."""
        if args:
            config.process_args(args)

        return config


    class SomeOtherPlugin:
        """Some other plugin defining the same hook."""

        @hookimpl(tryfirst=True)
        def setup_project(self, config, args):
            """Report what args were passed before calling
            downstream hooks.
            """
            if args:
                print("Got args: {}".format(args))

            return config


    pm = PluginManager("myproject")

    # load from the local module's namespace
    pm.register(sys.modules[__name__])
    # load a plugin defined on a class
    pm.register(SomeOtherPlugin())

For another example see the :ref:`pytest:plugin-hookorder` section of the
``pytest`` docs.

.. note::
    ``tryfirst`` and ``trylast`` hooks are still invoked in LIFO order within
    each category.


.. _hookwrappers:

Wrappers
^^^^^^^^
A *hookimpl* can be marked with a ``"hookwrapper"`` option which indicates that
the function will be called to *wrap* (or surround) all other normal *hookimpl*
calls. A *hookwrapper* can thus execute some code ahead and after the execution
of all corresponding non-wrappper *hookimpls*.

Much in the same way as a :py:func:`@contextlib.contextmanager <python:contextlib.contextmanager>`, *hookwrappers* must
be implemented as generator function with a single ``yield`` in its body:


.. code-block:: python

    @hookimpl(hookwrapper=True)
    def setup_project(config, args):
        """Wrap calls to ``setup_project()`` implementations which
        should return json encoded config options.
        """
        if config.debug:
            print("Pre-hook config is {}".format(config.tojson()))

        # get initial default config
        defaults = config.tojson()

        # all corresponding hookimpls are invoked here
        outcome = yield

        for item in outcome.get_result():
            print("JSON config override is {}".format(item))

        if config.debug:
            print("Post-hook config is {}".format(config.tojson()))

        if config.use_defaults:
            outcome.force_result(defaults)

The generator is :py:meth:`sent <python:generator.send>` a :py:class:`pluggy._callers._Result` object which can
be assigned in the ``yield`` expression and used to override or inspect
the final result(s) returned back to the caller using the
:py:meth:`~pluggy._callers._Result.force_result` or
:py:meth:`~pluggy._callers._Result.get_result` methods.

.. note::
    Hook wrappers can **not** return results (as per generator function
    semantics); they can only modify them using the ``_Result`` API.

Also see the :ref:`pytest:hookwrapper` section in the ``pytest`` docs.

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

    hookspec = HookspecMarker("myproject")


    @hookspec
    def setup_project(config, args):
        """This hook is used to process the initial config and input
        arguments.
        """


    pm = PluginManager("myproject")

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

.. _enforcing:

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

.. note::
    The one exception to this rule (that a *hookspec* must have as least as
    many arguments as its *hookimpls*) is the conventional :ref:`self <python:tut-remarks>` arg; this
    is always ignored when *hookimpls* are defined as :ref:`methods <python:tut-methodobjects>`.

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
:func:`~_pytest.hookspec.pytest_cmdline_main` central routine of ``pytest``.
Note that all ``hookwrappers`` are still invoked with the first result.

Also see the :ref:`pytest:firstresult` section in the ``pytest`` docs.

.. _historic:

Historic hooks
^^^^^^^^^^^^^^
You can mark a *hookspec* as being *historic* meaning that the hook
can be called with :py:meth:`~pluggy._hooks._HookCaller.call_historic()` **before**
having been registered:

.. code-block:: python

    @hookspec(historic=True)
    def myhook(config, args):
        pass

The implication is that late registered *hookimpls* will be called back
immediately at register time and **can not** return a result to the caller.

This turns out to be particularly useful when dealing with lazy or
dynamically loaded plugins.

For more info see :ref:`call_historic`.


Warnings on hook implementation
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

As projects evolve new hooks may be introduced and/or deprecated.

if a hookspec specifies a ``warn_on_impl``, pluggy will trigger it for any plugin implementing the hook.


.. code-block:: python

    @hookspec(
        warn_on_impl=DeprecationWarning("oldhook is deprecated and will be removed soon")
    )
    def oldhook():
        pass

.. _manage:

The Plugin registry
*******************
``pluggy`` manages plugins using instances of the
:py:class:`pluggy.PluginManager`.

A :py:class:`~pluggy.PluginManager` is instantiated with a single
``str`` argument, the ``project_name``:

.. code-block:: python

    import pluggy

    pm = pluggy.PluginManager("my_project_name")


The ``project_name`` value is used when a :py:class:`~pluggy.PluginManager`
scans for *hook* functions :ref:`defined on a plugin <define>`.
This allows for multiple plugin managers from multiple projects
to define hooks alongside each other.

.. _registration:

Registration
------------
Each :py:class:`~pluggy.PluginManager` maintains a *plugin* registry where each *plugin*
contains a set of *hookimpl* definitions. Loading *hookimpl* and *hookspec*
definitions to populate the registry is described in detail in the section on
:ref:`define`.

In summary, you pass a plugin namespace object to the
:py:meth:`~pluggy.PluginManager.register()` and
:py:meth:`~pluggy.PluginManager.add_hookspecs()` methods to collect
hook *implementations* and *specifications* from *plugin* namespaces respectively.

You can unregister any *plugin*'s hooks using
:py:meth:`~pluggy.PluginManager.unregister()` and check if a plugin is
registered by passing its name to the
:py:meth:`~pluggy.PluginManager.is_registered()` method.

Loading ``setuptools`` entry points
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
You can automatically load plugins registered through
:ref:`setuptools entry points <setuptools:entry_points>`
with the :py:meth:`~pluggy.PluginManager.load_setuptools_entrypoints()`
method.

An example use of this is the :ref:`pytest entry point <pytest:pip-installable plugins>`.


Blocking
--------
You can block any plugin from being registered using
:py:meth:`~pluggy.PluginManager.set_blocked()` and check if a given
*plugin* is blocked by name using :py:meth:`~pluggy.PluginManager.is_blocked()`.


Inspection
----------
You can use a variety of methods to inspect both the registry
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


.. _calling:

Calling hooks
*************
The core functionality of ``pluggy`` enables an extension provider
to override function calls made at certain points throughout a program.

A particular *hook* is invoked by calling an instance of
a :py:class:`pluggy._hooks._HookCaller` which in turn *loops* through the
``1:N`` registered *hookimpls* and calls them in sequence.

Every :py:class:`~pluggy.PluginManager` has a ``hook`` attribute
which is an instance of this :py:class:`pluggy._hooks._HookRelay`.
The :py:class:`~pluggy._hooks._HookRelay` itself contains references
(by hook name) to each registered *hookimpl*'s :py:class:`~pluggy._hooks._HookCaller` instance.

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

Note that you **must** call hooks using keyword :std:term:`python:argument` syntax!

Hook implementations are called in LIFO registered order: *the last
registered plugin's hooks are called first*. As an example, the below
assertion should not error:

.. code-block:: python

    from pluggy import PluginManager, HookimplMarker

    hookimpl = HookimplMarker("myproject")


    class Plugin1:
        @hookimpl
        def myhook(self, args):
            """Default implementation."""
            return 1


    class Plugin2:
        @hookimpl
        def myhook(self, args):
            """Default implementation."""
            return 2


    class Plugin3:
        @hookimpl
        def myhook(self, args):
            """Default implementation."""
            return 3


    pm = PluginManager("myproject")
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
its :ref:`first result only <firstresult>` in which case only the first
single value (which is not ``None``) will be returned.

.. _call_historic:

Exception handling
------------------
If any *hookimpl* errors with an exception no further callbacks
are invoked and the exception is packaged up and delivered to
any :ref:`wrappers <hookwrappers>` before being re-raised at the
hook invocation point:

.. code-block:: python

    from pluggy import PluginManager, HookimplMarker

    hookimpl = HookimplMarker("myproject")


    class Plugin1:
        @hookimpl
        def myhook(self, args):
            return 1


    class Plugin2:
        @hookimpl
        def myhook(self, args):
            raise RuntimeError


    class Plugin3:
        @hookimpl
        def myhook(self, args):
            return 3


    @hookimpl(hookwrapper=True)
    def myhook(self, args):
        outcome = yield

        try:
            outcome.get_result()
        except RuntimeError:
            # log the error details
            print(outcome.excinfo)


    pm = PluginManager("myproject")

    # register plugins
    pm.register(Plugin1())
    pm.register(Plugin2())
    pm.register(Plugin3())

    # register wrapper
    pm.register(sys.modules[__name__])

    # this raises RuntimeError due to Plugin2
    pm.hook.myhook(args=())

Historic calls
--------------
A *historic call* allows for all newly registered functions to receive all hook
calls that happened before their registration. The implication is that this is
only useful if you expect that some *hookimpls* may be registered **after** the
hook is initially invoked.

Historic hooks must be :ref:`specially marked <historic>` and called
using the :py:meth:`~pluggy._hooks._HookCaller.call_historic()` method:

.. code-block:: python

    def callback(result):
        print("historic call result is {result}".format(result=result))


    # call with history; no results returned
    pm.hook.myhook.call_historic(
        kwargs={"config": config, "args": sys.argv}, result_callback=callback
    )

    # ... more of our program ...

    # late loading of some plugin
    import mylateplugin

    # historic callback is invoked here
    pm.register(mylateplugin)

Note that if you :py:meth:`~pluggy._hooks._HookCaller.call_historic()`
the :py:class:`~pluggy._hooks._HookCaller` (and thus your calling code)
can not receive results back from the underlying *hookimpl* functions.
Instead you can provide a *callback* for processing results (like the
``callback`` function above) which will be called as each new plugin
is registered.

.. note::
    *historic* calls are incompatible with :ref:`firstresult` marked
    hooks since only the first registered plugin's hook(s) would
    ever be called.

Calling with extras
-------------------
You can call a hook with temporarily participating *implementation* functions
(that aren't in the registry) using the
:py:meth:`pluggy._hooks._HookCaller.call_extra()` method.


Calling with a subset of registered plugins
-------------------------------------------
You can make a call using a subset of plugins by asking the
:py:class:`~pluggy.PluginManager` first for a
:py:class:`~pluggy._hooks._HookCaller` with those plugins removed
using the :py:meth:`pluggy.PluginManager.subset_hook_caller()` method.

You then can use that :py:class:`_HookCaller <pluggy._hooks._HookCaller>`
to make normal, :py:meth:`~pluggy._hooks._HookCaller.call_historic`, or
:py:meth:`~pluggy._hooks._HookCaller.call_extra` calls as necessary.

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

    pm = PluginManager("myproject")
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

For development, we suggest to create a virtual environment and install ``pluggy`` in
editable mode and ``dev`` dependencies::

    $ python3 -m venv .env
    $ source .env/bin/activate
    $ pip install -e .[dev]

To make sure you follow the code style used in the project, install pre-commit_ which
will run style checks before each commit::

    $ pre-commit install


Release Policy
**************
Pluggy uses `Semantic Versioning`_. Breaking changes are only foreseen for
Major releases (incremented X in "X.Y.Z").  If you want to use ``pluggy``
in your project you should thus use a dependency restriction like
``"pluggy>=0.1.0,<1.0"`` to avoid surprises.


Table of contents
*****************

.. toctree::
    :maxdepth: 2

    api_reference
    changelog



.. hyperlinks
.. _hookspec module:
    https://docs.pytest.org/en/latest/_modules/_pytest/hookspec.html
.. _request-response pattern:
    https://en.wikipedia.org/wiki/Request%E2%80%93response
.. _publish-subscribe:
    https://en.wikipedia.org/wiki/Publish%E2%80%93subscribe_pattern
.. _hooking:
    https://en.wikipedia.org/wiki/Hooking
.. _callbacks:
    https://en.wikipedia.org/wiki/Callback_(computer_programming)
.. _tox test suite:
    https://github.com/pytest-dev/pluggy/blob/master/tox.ini
.. _Semantic Versioning:
    https://semver.org/
.. _Python interpreters:
    https://github.com/pytest-dev/pluggy/blob/master/tox.ini#L2
.. _500+ plugins:
    http://plugincompat.herokuapp.com/
.. _pre-commit:
    https://pre-commit.com/


.. Indices and tables
.. ==================
.. * :ref:`genindex`
.. * :ref:`modindex`
.. * :ref:`search`
