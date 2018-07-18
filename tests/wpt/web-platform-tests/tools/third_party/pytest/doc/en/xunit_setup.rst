
.. _`classic xunit`:
.. _xunitsetup:

classic xunit-style setup
========================================

This section describes a classic and popular way how you can implement
fixtures (setup and teardown test state) on a per-module/class/function basis.


.. note::

    While these setup/teardown methods are simple and familiar to those
    coming from a ``unittest`` or nose ``background``, you may also consider
    using pytest's more powerful :ref:`fixture mechanism
    <fixture>` which leverages the concept of dependency injection, allowing
    for a more modular and more scalable approach for managing test state,
    especially for larger projects and for functional testing.  You can
    mix both fixture mechanisms in the same file but
    test methods of ``unittest.TestCase`` subclasses
    cannot receive fixture arguments.


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

As of pytest-3.0, the ``module`` parameter is optional.

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

As of pytest-3.0, the ``method`` parameter is optional.

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

As of pytest-3.0, the ``function`` parameter is optional.

Remarks:

* It is possible for setup/teardown pairs to be invoked multiple times
  per testing process.
* teardown functions are not called if the corresponding setup function existed
  and failed/was skipped.

.. _`unittest.py module`: http://docs.python.org/library/unittest.html
