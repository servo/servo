
.. _`classic xunit`:
.. _xunitsetup:

classic xunit-style setup
========================================

This section describes a classic and popular way how you can implement
fixtures (setup and teardown test state) on a per-module/class/function basis.  
pytest started supporting these methods around 2005 and subsequently
nose and the standard library introduced them (under slightly different
names).  While these setup/teardown methods are and will remain fully
supported you may also use pytest's more powerful :ref:`fixture mechanism
<fixture>` which leverages the concept of dependency injection, allowing
for a more modular and more scalable approach for managing test state, 
especially for larger projects and for functional testing.  You can
mix both fixture mechanisms in the same file but unittest-based
test methods cannot receive fixture arguments.

.. note::

    As of pytest-2.4, teardownX functions are not called if 
    setupX existed and failed/was skipped.  This harmonizes
    behaviour across all major python testing tools.

Module level setup/teardown
--------------------------------------

If you have multiple test functions and test classes in a single
module you can optionally implement the following fixture methods
which will usually be called once for all the functions::

    def setup_module(module):
        """ setup any state specific to the execution of the given module."""

    def teardown_module(module):
        """ teardown any state that was previously setup with a setup_module
        method.
        """

Class level setup/teardown
----------------------------------

Similarly, the following methods are called at class level before
and after all test methods of the class are called::

    @classmethod
    def setup_class(cls):
        """ setup any state specific to the execution of the given class (which
        usually contains tests).
        """

    @classmethod
    def teardown_class(cls):
        """ teardown any state that was previously setup with a call to
        setup_class.
        """

Method and function level setup/teardown
-----------------------------------------------

Similarly, the following methods are called around each method invocation::

    def setup_method(self, method):
        """ setup any state tied to the execution of the given method in a
        class.  setup_method is invoked for every test method of a class.
        """

    def teardown_method(self, method):
        """ teardown any state that was previously setup with a setup_method
        call.
        """

If you would rather define test functions directly at module level
you can also use the following functions to implement fixtures::

    def setup_function(function):
        """ setup any state tied to the execution of the given function.
        Invoked for every test function in the module.
        """

    def teardown_function(function):
        """ teardown any state that was previously setup with a setup_function
        call.
        """

Note that it is possible for setup/teardown pairs to be invoked multiple times
per testing process.

.. _`unittest.py module`: http://docs.python.org/library/unittest.html
