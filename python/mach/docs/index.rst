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

.. toctree::
   :maxdepth: 1

   commands
   driver
   logging
