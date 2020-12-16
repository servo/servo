
Doctest integration for modules and test files
=========================================================

By default all files matching the ``test*.txt`` pattern will
be run through the python standard ``doctest`` module.  You
can change the pattern by issuing:

.. code-block:: bash

    pytest --doctest-glob='*.rst'

on the command line. ``--doctest-glob`` can be given multiple times in the command-line.

If you then have a text file like this:

.. code-block:: text

    # content of test_example.txt

    hello this is a doctest
    >>> x = 3
    >>> x
    3

then you can just invoke ``pytest`` directly:

.. code-block:: pytest

    $ pytest
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-4.x.y, py-1.x.y, pluggy-0.x.y
    cachedir: $PYTHON_PREFIX/.pytest_cache
    rootdir: $REGENDOC_TMPDIR
    collected 1 item

    test_example.txt .                                                   [100%]

    ========================= 1 passed in 0.12 seconds =========================

By default, pytest will collect ``test*.txt`` files looking for doctest directives, but you
can pass additional globs using the ``--doctest-glob`` option (multi-allowed).

In addition to text files, you can also execute doctests directly from docstrings of your classes
and functions, including from test modules:

.. code-block:: python

    # content of mymodule.py
    def something():
        """ a doctest in a docstring
        >>> something()
        42
        """
        return 42

.. code-block:: bash

    $ pytest --doctest-modules
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-4.x.y, py-1.x.y, pluggy-0.x.y
    cachedir: $PYTHON_PREFIX/.pytest_cache
    rootdir: $REGENDOC_TMPDIR
    collected 2 items

    mymodule.py .                                                        [ 50%]
    test_example.txt .                                                   [100%]

    ========================= 2 passed in 0.12 seconds =========================

You can make these changes permanent in your project by
putting them into a pytest.ini file like this:

.. code-block:: ini

    # content of pytest.ini
    [pytest]
    addopts = --doctest-modules

.. note::

    The builtin pytest doctest supports only ``doctest`` blocks, but if you are looking
    for more advanced checking over *all* your documentation,
    including doctests, ``.. codeblock:: python`` Sphinx directive support,
    and any other examples your documentation may include, you may wish to
    consider `Sybil <https://sybil.readthedocs.io/en/latest/index.html>`__.
    It provides pytest integration out of the box.


Encoding
--------

The default encoding is **UTF-8**, but you can specify the encoding
that will be used for those doctest files using the
``doctest_encoding`` ini option:

.. code-block:: ini

    # content of pytest.ini
    [pytest]
    doctest_encoding = latin1

Using 'doctest' options
-----------------------

The standard ``doctest`` module provides some `options <https://docs.python.org/3/library/doctest.html#option-flags>`__
to configure the strictness of doctest tests. In pytest, you can enable those flags using the
configuration file.

For example, to make pytest ignore trailing whitespaces and ignore
lengthy exception stack traces you can just write:

.. code-block:: ini

    [pytest]
    doctest_optionflags= NORMALIZE_WHITESPACE IGNORE_EXCEPTION_DETAIL

pytest also introduces new options to allow doctests to run in Python 2 and
Python 3 unchanged:

* ``ALLOW_UNICODE``: when enabled, the ``u`` prefix is stripped from unicode
  strings in expected doctest output.

* ``ALLOW_BYTES``: when enabled, the ``b`` prefix is stripped from byte strings
  in expected doctest output.

Alternatively, options can be enabled by an inline comment in the doc test
itself:

.. code-block:: rst

    # content of example.rst
    >>> get_unicode_greeting()  # doctest: +ALLOW_UNICODE
    'Hello'

By default, pytest would report only the first failure for a given doctest. If
you want to continue the test even when you have failures, do:

.. code-block:: bash

    pytest --doctest-modules --doctest-continue-on-failure


Output format
-------------

You can change the diff output format on failure for your doctests
by using one of standard doctest modules format in options
(see :data:`python:doctest.REPORT_UDIFF`, :data:`python:doctest.REPORT_CDIFF`,
:data:`python:doctest.REPORT_NDIFF`, :data:`python:doctest.REPORT_ONLY_FIRST_FAILURE`):

.. code-block:: bash

    pytest --doctest-modules --doctest-report none
    pytest --doctest-modules --doctest-report udiff
    pytest --doctest-modules --doctest-report cdiff
    pytest --doctest-modules --doctest-report ndiff
    pytest --doctest-modules --doctest-report only_first_failure


pytest-specific features
------------------------

Some features are provided to make writing doctests easier or with better integration with
your existing test suite. Keep in mind however that by using those features you will make
your doctests incompatible with the standard ``doctests`` module.

Using fixtures
^^^^^^^^^^^^^^

It is possible to use fixtures using the ``getfixture`` helper:

.. code-block:: text

    # content of example.rst
    >>> tmp = getfixture('tmpdir')
    >>> ...
    >>>

Also, :ref:`usefixtures` and :ref:`autouse` fixtures are supported
when executing text doctest files.


.. _`doctest_namespace`:

'doctest_namespace' fixture
^^^^^^^^^^^^^^^^^^^^^^^^^^^

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

Note that like the normal ``conftest.py``, the fixtures are discovered in the directory tree conftest is in.
Meaning that if you put your doctest with your source code, the relevant conftest.py needs to be in the same directory tree.
Fixtures will not be discovered in a sibling directory tree!

Skipping tests dynamically
^^^^^^^^^^^^^^^^^^^^^^^^^^

.. versionadded:: 4.4

You can use ``pytest.skip`` to dynamically skip doctests. For example::

    >>> import sys, pytest
    >>> if sys.platform.startswith('win'):
    ...     pytest.skip('this doctest does not work on Windows')
    ...
