====
mach
====

Mach (German for *do*) is a generic command dispatcher for the command
line.

To use mach, you install the mach core (a Python package), create an
executable *driver* script (named whatever you want), and write mach
commands. When the *driver* is executed, mach dispatches to the
requested command handler automatically.

Features
========

On a high level, mach is similar to using argparse with subparsers (for
command handling). When you dig deeper, mach offers a number of
additional features:

Distributed command definitions
  With optparse/argparse, you have to define your commands on a central
  parser instance. With mach, you annotate your command methods with
  decorators and mach finds and dispatches to them automatically.

Command categories
  Mach commands can be grouped into categories when displayed in help.
  This is currently not possible with argparse.

Logging management
  Mach provides a facility for logging (both classical text and
  structured) that is available to any command handler.

Settings files
  Mach provides a facility for reading settings from an ini-like file
  format.

Components
==========

Mach is conceptually composed of the following components:

core
  The mach core is the core code powering mach. This is a Python package
  that contains all the business logic that makes mach work. The mach
  core is common to all mach deployments.

commands
  These are what mach dispatches to. Commands are simply Python methods
  registered as command names. The set of commands is unique to the
  environment mach is deployed in.

driver
  The *driver* is the entry-point to mach. It is simply an executable
  script that loads the mach core, tells it where commands can be found,
  then asks the mach core to handle the current request. The driver is
  unique to the deployed environment. But, it's usually based on an
  example from this source tree.

Project State
=============

mach was originally written as a command dispatching framework to aid
Firefox development. While the code is mostly generic, there are still
some pieces that closely tie it to Mozilla/Firefox. The goal is for
these to eventually be removed and replaced with generic features so
mach is suitable for anybody to use. Until then, mach may not be the
best fit for you.

Implementing Commands
---------------------

Mach commands are defined via Python decorators.

All the relevant decorators are defined in the *mach.decorators* module.
The important decorators are as follows:

CommandProvider
  A class decorator that denotes that a class contains mach
  commands. The decorator takes no arguments.

Command
  A method decorator that denotes that the method should be called when
  the specified command is requested. The decorator takes a command name
  as its first argument and a number of additional arguments to
  configure the behavior of the command.

CommandArgument
  A method decorator that defines an argument to the command. Its
  arguments are essentially proxied to ArgumentParser.add_argument()

Classes with the *@CommandProvider* decorator *must* have an *__init__*
method that accepts 1 or 2 arguments. If it accepts 2 arguments, the
2nd argument will be a *MachCommandContext* instance. This is just a named
tuple containing references to objects provided by the mach driver.

Here is a complete example::

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
*mach.base* module for more.

The Python modules defining mach commands do not need to live inside the
main mach source tree.

Conditionally Filtering Commands
--------------------------------

Sometimes it might only make sense to run a command given a certain
context. For example, running tests only makes sense if the product
they are testing has been built, and said build is available. To make
sure a command is only runnable from within a correct context, you can
define a series of conditions on the *Command* decorator.

A condition is simply a function that takes an instance of the
*CommandProvider* class as an argument, and returns True or False. If
any of the conditions defined on a command return False, the command
will not be runnable. The doc string of a condition function is used in
error messages, to explain why the command cannot currently be run.

Here is an example:

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
but it is possible to change this behaviour by setting *require_conditions*
to True:

    m = mach.main.Mach()
    m.require_conditions = True

Minimizing Code in Commands
---------------------------

Mach command modules, classes, and methods work best when they are
minimal dispatchers. The reason is import bloat. Currently, the mach
core needs to import every Python file potentially containing mach
commands for every command invocation. If you have dozens of commands or
commands in modules that import a lot of Python code, these imports
could slow mach down and waste memory.

It is thus recommended that mach modules, classes, and methods do as
little work as possible. Ideally the module should only import from
the *mach* package. If you need external modules, you should import them
from within the command method.

To keep code size small, the body of a command method should be limited
to:

1. Obtaining user input (parsing arguments, prompting, etc)
2. Calling into some other Python package
3. Formatting output

Of course, these recommendations can be ignored if you want to risk
slower performance.

In the future, the mach driver may cache the dispatching information or
have it intelligently loaded to facilitate lazy loading.

Logging
=======

Mach configures a built-in logging facility so commands can easily log
data.

What sets the logging facility apart from most loggers you've seen is
that it encourages structured logging. Instead of conventional logging
where simple strings are logged, the internal logging mechanism logs all
events with the following pieces of information:

* A string *action*
* A dict of log message fields
* A formatting string

Essentially, instead of assembling a human-readable string at
logging-time, you create an object holding all the pieces of data that
will constitute your logged event. For each unique type of logged event,
you assign an *action* name.

Depending on how logging is configured, your logged event could get
written a couple of different ways.

JSON Logging
------------

Where machines are the intended target of the logging data, a JSON
logger is configured. The JSON logger assembles an array consisting of
the following elements:

* Decimal wall clock time in seconds since UNIX epoch
* String *action* of message
* Object with structured message data

The JSON-serialized array is written to a configured file handle.
Consumers of this logging stream can just perform a readline() then feed
that into a JSON deserializer to reconstruct the original logged
message. They can key off the *action* element to determine how to
process individual events. There is no need to invent a parser.
Convenient, isn't it?

Logging for Humans
------------------

Where humans are the intended consumer of a log message, the structured
log message are converted to more human-friendly form. This is done by
utilizing the *formatting* string provided at log time. The logger
simply calls the *format* method of the formatting string, passing the
dict containing the message's fields.

When *mach* is used in a terminal that supports it, the logging facility
also supports terminal features such as colorization. This is done
automatically in the logging layer - there is no need to control this at
logging time.

In addition, messages intended for humans typically prepends every line
with the time passed since the application started.

Logging HOWTO
-------------

Structured logging piggybacks on top of Python's built-in logging
infrastructure provided by the *logging* package. We accomplish this by
taking advantage of *logging.Logger.log()*'s *extra* argument. To this
argument, we pass a dict with the fields *action* and *params*. These
are the string *action* and dict of message fields, respectively. The
formatting string is passed as the *msg* argument, like normal.

If you were logging to a logger directly, you would do something like:

    logger.log(logging.INFO, 'My name is {name}',
        extra={'action': 'my_name', 'params': {'name': 'Gregory'}})

The JSON logging would produce something like:

    [1339985554.306338, "my_name", {"name": "Gregory"}]

Human logging would produce something like:

     0.52 My name is Gregory

Since there is a lot of complexity using logger.log directly, it is
recommended to go through a wrapping layer that hides part of the
complexity for you. The easiest way to do this is by utilizing the
LoggingMixin:

    import logging
    from mach.mixin.logging import LoggingMixin

    class MyClass(LoggingMixin):
        def foo(self):
             self.log(logging.INFO, 'foo_start', {'bar': True},
                 'Foo performed. Bar: {bar}')

Entry Points
============

It is possible to use setuptools' entry points to load commands
directly from python packages. A mach entry point is a function which
returns a list of files or directories containing mach command
providers. e.g.::

    def list_providers():
        providers = []
        here = os.path.abspath(os.path.dirname(__file__))
        for p in os.listdir(here):
            if p.endswith('.py'):
                providers.append(os.path.join(here, p))
        return providers

See http://pythonhosted.org/setuptools/setuptools.html#dynamic-discovery-of-services-and-plugins
for more information on creating an entry point. To search for entry
point plugins, you can call *load_commands_from_entry_point*. This
takes a single parameter called *group*. This is the name of the entry
point group to load and defaults to ``mach.providers``. e.g.::

    mach.load_commands_from_entry_point("mach.external.providers")

Adding Global Arguments
=======================

Arguments to mach commands are usually command-specific. However,
mach ships with a handful of global arguments that apply to all
commands.

It is possible to extend the list of global arguments. In your
*mach driver*, simply call ``add_global_argument()`` on your
``mach.main.Mach`` instance. e.g.::

   mach = mach.main.Mach(os.getcwd())

   # Will allow --example to be specified on every mach command.
   mach.add_global_argument('--example', action='store_true',
       help='Demonstrate an example global argument.')
