
Doctest integration for modules and test files
=========================================================

By default all files matching the ``test*.txt`` pattern will
be run through the python standard ``doctest`` module.  You
can change the pattern by issuing::

    pytest --doctest-glob='*.rst'

on the command line. Since version ``2.9``, ``--doctest-glob``
can be given multiple times in the command-line.

.. versionadded:: 3.1

    You can specify the encoding that will be used for those doctest files
    using the ``doctest_encoding`` ini option:

    .. code-block:: ini

        # content of pytest.ini
        [pytest]
        doctest_encoding = latin1

    The default encoding is UTF-8.

You can also trigger running of doctests
from docstrings in all python modules (including regular
python test modules)::

    pytest --doctest-modules

You can make these changes permanent in your project by
putting them into a pytest.ini file like this:

.. code-block:: ini

    # content of pytest.ini
    [pytest]
    addopts = --doctest-modules

If you then have a text file like this::

    # content of example.rst

    hello this is a doctest
    >>> x = 3
    >>> x
    3

and another like this::

    # content of mymodule.py
    def something():
        """ a doctest in a docstring
        >>> something()
        42
        """
        return 42

then you can just invoke ``pytest`` without command line options::

    $ pytest
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-3.x.y, py-1.x.y, pluggy-0.x.y
    rootdir: $REGENDOC_TMPDIR, inifile: pytest.ini
    collected 1 item
    
    mymodule.py .                                                        [100%]
    
    ========================= 1 passed in 0.12 seconds =========================

It is possible to use fixtures using the ``getfixture`` helper::

    # content of example.rst
    >>> tmp = getfixture('tmpdir')
    >>> ...
    >>>

Also, :ref:`usefixtures` and :ref:`autouse` fixtures are supported
when executing text doctest files.

The standard ``doctest`` module provides some setting flags to configure the
strictness of doctest tests. In pytest You can enable those flags those flags
using the configuration file. To make pytest ignore trailing whitespaces and
ignore lengthy exception stack traces you can just write:

.. code-block:: ini

    [pytest]
    doctest_optionflags= NORMALIZE_WHITESPACE IGNORE_EXCEPTION_DETAIL

pytest also introduces new options to allow doctests to run in Python 2 and
Python 3 unchanged:

* ``ALLOW_UNICODE``: when enabled, the ``u`` prefix is stripped from unicode
  strings in expected doctest output.

* ``ALLOW_BYTES``: when enabled, the ``b`` prefix is stripped from byte strings
  in expected doctest output.

As with any other option flag, these flags can be enabled in ``pytest.ini`` using
the ``doctest_optionflags`` ini option:

.. code-block:: ini

    [pytest]
    doctest_optionflags = ALLOW_UNICODE ALLOW_BYTES


Alternatively, it can be enabled by an inline comment in the doc test
itself::

    # content of example.rst
    >>> get_unicode_greeting()  # doctest: +ALLOW_UNICODE
    'Hello'


The 'doctest_namespace' fixture
-------------------------------

.. versionadded:: 3.0

The ``doctest_namespace`` fixture can be used to inject items into the
namespace in which your doctests run. It is intended to be used within
your own fixtures to provide the tests that use them with context.

``doctest_namespace`` is a standard ``dict`` object into which you
place the objects you want to appear in the doctest namespace::

    # content of conftest.py
    import numpy
    @pytest.fixture(autouse=True)
    def add_np(doctest_namespace):
        doctest_namespace['np'] = numpy

which can then be used in your doctests directly::

    # content of numpy.py
    def arange():
        """
        >>> a = np.arange(10)
        >>> len(a)
        10
        """
        pass


Output format
-------------

.. versionadded:: 3.0

You can change the diff output format on failure for your doctests
by using one of standard doctest modules format in options
(see :data:`python:doctest.REPORT_UDIFF`, :data:`python:doctest.REPORT_CDIFF`,
:data:`python:doctest.REPORT_NDIFF`, :data:`python:doctest.REPORT_ONLY_FIRST_FAILURE`)::

    pytest --doctest-modules --doctest-report none
    pytest --doctest-modules --doctest-report udiff
    pytest --doctest-modules --doctest-report cdiff
    pytest --doctest-modules --doctest-report ndiff
    pytest --doctest-modules --doctest-report only_first_failure


