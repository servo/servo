.. _mach_driver:

=======
Drivers
=======

Entry Points
============

It is possible to use setuptools' entry points to load commands
directly from python packages. A mach entry point is a function which
returns a list of files or directories containing mach command
providers. e.g.:

.. code-block:: python

   def list_providers():
       providers = []
       here = os.path.abspath(os.path.dirname(__file__))
       for p in os.listdir(here):
           if p.endswith('.py'):
               providers.append(os.path.join(here, p))
       return providers

See http://pythonhosted.org/setuptools/setuptools.html#dynamic-discovery-of-services-and-plugins
for more information on creating an entry point. To search for entry
point plugins, you can call
:py:meth:`mach.main.Mach.load_commands_from_entry_point`. e.g.:

.. code-block:: python

   mach.load_commands_from_entry_point("mach.external.providers")

Adding Global Arguments
=======================

Arguments to mach commands are usually command-specific. However,
mach ships with a handful of global arguments that apply to all
commands.

It is possible to extend the list of global arguments. In your
*mach driver*, simply call
:py:meth:`mach.main.Mach.add_global_argument`. e.g.:

.. code-block:: python

   mach = mach.main.Mach(os.getcwd())

   # Will allow --example to be specified on every mach command.
   mach.add_global_argument('--example', action='store_true',
       help='Demonstrate an example global argument.')
