.. _mach_commands:

=====================
Implementing Commands
=====================

Mach commands are defined via Python decorators.

All the relevant decorators are defined in the *mach.decorators* module.
The important decorators are as follows:

:py:func:`CommandProvider <mach.decorators.CommandProvider>`
  A class decorator that denotes that a class contains mach
  commands. The decorator takes no arguments.

:py:func:`Command <mach.decorators.Command>`
  A method decorator that denotes that the method should be called when
  the specified command is requested. The decorator takes a command name
  as its first argument and a number of additional arguments to
  configure the behavior of the command.

:py:func:`CommandArgument <mach.decorators.CommandArgument>`
  A method decorator that defines an argument to the command. Its
  arguments are essentially proxied to ArgumentParser.add_argument()

:py:func:`SubCommand <mach.decorators.SubCommand>`
  A method decorator that denotes that the method should be a
  sub-command to an existing ``@Command``. The decorator takes the
  parent command name as its first argument and the sub-command name
  as its second argument.

  ``@CommandArgument`` can be used on ``@SubCommand`` instances just
  like they can on ``@Command`` instances.

Classes with the ``@CommandProvider`` decorator **must** have an
``__init__`` method that accepts 1 or 2 arguments. If it accepts 2
arguments, the 2nd argument will be a
:py:class:`mach.base.CommandContext` instance.

Here is a complete example:

.. code-block:: python

   from mach.decorators import (
       CommandArgument,
       CommandProvider,
       Command,
   )

   @CommandProvider
   class MyClass(object):
       @Command('doit', help='Do ALL OF THE THINGS.')
       @CommandArgument('--force', '-f', action='store_true',
           help='Force doing it.')
       def doit(self, force=False):
           # Do stuff here.

When the module is loaded, the decorators tell mach about all handlers.
When mach runs, it takes the assembled metadata from these handlers and
hooks it up to the command line driver. Under the hood, arguments passed
to the decorators are being used to help mach parse command arguments,
formulate arguments to the methods, etc. See the documentation in the
:py:mod:`mach.base` module for more.

The Python modules defining mach commands do not need to live inside the
main mach source tree.

Conditionally Filtering Commands
================================

Sometimes it might only make sense to run a command given a certain
context. For example, running tests only makes sense if the product
they are testing has been built, and said build is available. To make
sure a command is only runnable from within a correct context, you can
define a series of conditions on the
:py:func:`Command <mach.decorators.Command>` decorator.

A condition is simply a function that takes an instance of the
:py:func:`mach.decorators.CommandProvider` class as an argument, and
returns ``True`` or ``False``. If any of the conditions defined on a
command return ``False``, the command will not be runnable. The
docstring of a condition function is used in error messages, to explain
why the command cannot currently be run.

Here is an example:

.. code-block:: python

   from mach.decorators import (
       CommandProvider,
       Command,
   )

   def build_available(cls):
       """The build needs to be available."""
       return cls.build_path is not None

    @CommandProvider
   class MyClass(MachCommandBase):
       def __init__(self, build_path=None):
           self.build_path = build_path

       @Command('run_tests', conditions=[build_available])
       def run_tests(self):
           # Do stuff here.

It is important to make sure that any state needed by the condition is
available to instances of the command provider.

By default all commands without any conditions applied will be runnable,
but it is possible to change this behaviour by setting
``require_conditions`` to ``True``:

.. code-block:: python

   m = mach.main.Mach()
   m.require_conditions = True

Minimizing Code in Commands
===========================

Mach command modules, classes, and methods work best when they are
minimal dispatchers. The reason is import bloat. Currently, the mach
core needs to import every Python file potentially containing mach
commands for every command invocation. If you have dozens of commands or
commands in modules that import a lot of Python code, these imports
could slow mach down and waste memory.

It is thus recommended that mach modules, classes, and methods do as
little work as possible. Ideally the module should only import from
the :py:mod:`mach` package. If you need external modules, you should
import them from within the command method.

To keep code size small, the body of a command method should be limited
to:

1. Obtaining user input (parsing arguments, prompting, etc)
2. Calling into some other Python package
3. Formatting output

Of course, these recommendations can be ignored if you want to risk
slower performance.

In the future, the mach driver may cache the dispatching information or
have it intelligently loaded to facilitate lazy loading.
