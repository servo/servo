===============================================
ATTIC documentation
===============================================

XXX REVIEW and remove the below  XXX

Customizing the testing process
===============================

writing conftest.py files
-----------------------------------

You may put conftest.py files containing project-specific
configuration in your project's root directory, it's usually
best to put it just into the same directory level as your
topmost ``__init__.py``.  In fact, ``pytest`` performs
an "upwards" search starting from the directory that you specify
to be tested and will lookup configuration values right-to-left.
You may have options that reside e.g. in your home directory
but note that project specific settings will be considered
first.  There is a flag that helps you debugging your
conftest.py configurations::

    py.test --traceconfig


customizing the collecting and running process
-----------------------------------------------

To introduce different test items you can create
one or more ``conftest.py`` files in your project.
When the collection process traverses directories
and modules the default collectors will produce
custom Collectors and Items if they are found
in a local ``conftest.py`` file.


Customizing the collection process in a module
----------------------------------------------

If you have a module where you want to take responsibility for
collecting your own test Items and possibly even for executing
a test then you can provide `generative tests`_ that yield
callables and possibly arguments as a tuple.   This is especially
useful for calling application test machinery with different
parameter sets but counting each of the calls as a separate
tests.

.. _`generative tests`: features.html#generative-tests

The other extension possibility is about
specifying a custom test ``Item`` class which
is responsible for setting up and executing an underlying
test.  Or you can extend the collection process for a whole
directory tree by putting Items in a ``conftest.py`` configuration file.
The collection process dynamically consults the *chain of conftest.py*
modules to determine collectors and items at ``Directory``, ``Module``,
``Class``, ``Function`` or ``Generator`` level respectively.

Customizing execution of Items and Functions
----------------------------------------------------

- ``pytest.Function`` test items control execution
  of a test function through its ``function.runtest()`` method.
  This method is responsible for performing setup and teardown
  ("Test Fixtures") for a test Function.

- ``Function.execute(target, *args)`` methods are invoked by
  the default ``Function.run()`` to actually execute a python
  function with the given (usually empty set of) arguments.

.. _`py-dev mailing list`: http://codespeak.net/mailman/listinfo/py-dev


.. _`test generators`: funcargs.html#test-generators

.. _`generative tests`:

generative tests: yielding parametrized tests
====================================================

Deprecated since 1.0 in favour of `test generators`_.

*Generative tests* are test methods that are *generator functions* which
``yield`` callables and their arguments.  This is useful for running a
test function multiple times against different parameters.  Example::

    def test_generative():
        for x in (42,17,49):
            yield check, x

    def check(arg):
        assert arg % 7 == 0   # second generated tests fails!

Note that ``test_generative()`` will cause three tests
to get run, notably ``check(42)``, ``check(17)`` and ``check(49)``
of which the middle one will obviously fail.

To make it easier to distinguish the generated tests it is possible to specify an explicit name for them, like for example::

    def test_generative():
        for x in (42,17,49):
            yield "case %d" % x, check, x


disabling a test class
----------------------

If you want to disable a complete test class you
can set the class-level attribute ``disabled``.
For example, in order to avoid running some tests on Win32::

    class TestPosixOnly:
        disabled = sys.platform == 'win32'

        def test_xxx(self):
            ...
