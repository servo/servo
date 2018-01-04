Some Issues and Questions
==================================

.. note::

    This FAQ is here only mostly for historic reasons.  Checkout
    `pytest Q&A at Stackoverflow <http://stackoverflow.com/search?q=pytest>`_
    for many questions and answers related to pytest and/or use
    :ref:`contact channels` to get help.

On naming, nosetests, licensing and magic
------------------------------------------------

How does pytest relate to nose and unittest?
+++++++++++++++++++++++++++++++++++++++++++++++++

``pytest`` and nose_ share basic philosophy when it comes
to running and writing Python tests.  In fact, you can run many tests
written for nose with ``pytest``.  nose_ was originally created
as a clone of ``pytest`` when ``pytest`` was in the ``0.8`` release
cycle.  Note that starting with pytest-2.0 support for running unittest
test suites is majorly improved.

how does pytest relate to twisted's trial?
++++++++++++++++++++++++++++++++++++++++++++++

Since some time ``pytest`` has builtin support for supporting tests
written using trial. It does not itself start a reactor, however,
and does not handle Deferreds returned from a test in pytest style.
If you are using trial's unittest.TestCase chances are that you can
just run your tests even if you return Deferreds.  In addition,
there also is a dedicated `pytest-twisted
<http://pypi.python.org/pypi/pytest-twisted>`_ plugin which allows you to
return deferreds from pytest-style tests, allowing the use of
:ref:`fixtures` and other features.

how does pytest work with Django?
++++++++++++++++++++++++++++++++++++++++++++++

In 2012, some work is going into the `pytest-django plugin <http://pypi.python.org/pypi/pytest-django>`_.  It substitutes the usage of Django's
``manage.py test`` and allows the use of all pytest features_ most of which
are not available from Django directly.

.. _features: features.html


What's this "magic" with pytest? (historic notes)
++++++++++++++++++++++++++++++++++++++++++++++++++++++++

Around 2007 (version ``0.8``) some people thought that ``pytest``
was using too much "magic".  It had been part of the `pylib`_ which
contains a lot of unrelated python library code.  Around 2010 there
was a major cleanup refactoring, which removed unused or deprecated code
and resulted in the new ``pytest`` PyPI package which strictly contains
only test-related code.  This release also brought a complete pluginification
such that the core is around 300 lines of code and everything else is
implemented in plugins.  Thus ``pytest`` today is a small, universally runnable
and customizable testing framework for Python.   Note, however, that
``pytest`` uses metaprogramming techniques and reading its source is
thus likely not something for Python beginners.

A second "magic" issue was the assert statement debugging feature.
Nowadays, ``pytest`` explicitly rewrites assert statements in test modules
in order to provide more useful :ref:`assert feedback <assertfeedback>`.
This completely avoids previous issues of confusing assertion-reporting.
It also means, that you can use Python's ``-O`` optimization without losing
assertions in test modules.

You can also turn off all assertion interaction using the
``--assert=plain`` option.

.. _`py namespaces`: index.html
.. _`py/__init__.py`: http://bitbucket.org/hpk42/py-trunk/src/trunk/py/__init__.py


Why can I use both ``pytest`` and ``py.test`` commands?
+++++++++++++++++++++++++++++++++++++++++++++++++++++++

pytest used to be part of the py package, which provided several developer
utilities, all starting with ``py.<TAB>``, thus providing nice TAB-completion.
If you install ``pip install pycmd`` you get these tools from a separate
package. Once ``pytest`` became a separate package, the ``py.test`` name was
retained due to avoid a naming conflict with another tool. This conflict was
eventually resolved, and the ``pytest`` command was therefore introduced. In
future versions of pytest, we may deprecate and later remove the ``py.test``
command to avoid perpetuating the confusion.

pytest fixtures, parametrized tests
-------------------------------------------------------

.. _funcargs: funcargs.html

Is using pytest fixtures versus xUnit setup a style question?
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

For simple applications and for people experienced with nose_ or
unittest-style test setup using `xUnit style setup`_ probably
feels natural.  For larger test suites, parametrized testing
or setup of complex test resources using fixtures_ may feel more natural.
Moreover, fixtures are ideal for writing advanced test support
code (like e.g. the monkeypatch_, the tmpdir_ or capture_ fixtures)
because the support code can register setup/teardown functions
in a managed class/module/function scope.

.. _monkeypatch: monkeypatch.html
.. _tmpdir: tmpdir.html
.. _capture: capture.html
.. _fixtures: fixture.html

.. _`why pytest_pyfuncarg__ methods?`:

.. _`Convention over Configuration`: http://en.wikipedia.org/wiki/Convention_over_Configuration

Can I yield multiple values from a fixture function?
++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

There are two conceptual reasons why yielding from a factory function
is not possible:

* If multiple factories yielded values there would
  be no natural place to determine the combination
  policy - in real-world examples some combinations
  often should not run.

* Calling factories for obtaining test function arguments
  is part of setting up and running a test.  At that
  point it is not possible to add new test calls to
  the test collection anymore.

However, with pytest-2.3 you can use the :ref:`@pytest.fixture` decorator
and specify ``params`` so that all tests depending on the factory-created
resource will run multiple times with different parameters.

You can also use the ``pytest_generate_tests`` hook to
implement the `parametrization scheme of your choice`_. See also
:ref:`paramexamples` for more examples.

.. _`parametrization scheme of your choice`: http://tetamap.wordpress.com/2009/05/13/parametrizing-python-tests-generalized/

pytest interaction with other packages
---------------------------------------------------

Issues with pytest, multiprocess and setuptools?
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++

On Windows the multiprocess package will instantiate sub processes
by pickling and thus implicitly re-import a lot of local modules.
Unfortunately, setuptools-0.6.11 does not ``if __name__=='__main__'``
protect its generated command line script.  This leads to infinite
recursion when running a test that instantiates Processes.

As of mid-2013, there shouldn't be a problem anymore when you
use the standard setuptools (note that distribute has been merged
back into setuptools which is now shipped directly with virtualenv).

.. include:: links.inc
