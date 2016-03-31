
Doctest integration for modules and test files
=========================================================

By default all files matching the ``test*.txt`` pattern will
be run through the python standard ``doctest`` module.  You
can change the pattern by issuing::

    py.test --doctest-glob='*.rst'

on the command line. Since version ``2.9``, ``--doctest-glob``
can be given multiple times in the command-line.

You can also trigger running of doctests
from docstrings in all python modules (including regular
python test modules)::

    py.test --doctest-modules

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

then you can just invoke ``py.test`` without command line options::

    $ py.test
    ======= test session starts ========
    platform linux -- Python 3.4.0, pytest-2.9.1, py-1.4.31, pluggy-0.3.1
    rootdir: $REGENDOC_TMPDIR, inifile: pytest.ini
    collected 1 items
    
    mymodule.py .
    
    ======= 1 passed in 0.12 seconds ========

It is possible to use fixtures using the ``getfixture`` helper::

    # content of example.rst
    >>> tmp = getfixture('tmpdir')
    >>> ...
    >>>

Also, :ref:`usefixtures` and :ref:`autouse` fixtures are supported
when executing text doctest files.

The standard ``doctest`` module provides some setting flags to configure the
strictness of doctest tests. In py.test You can enable those flags those flags
using the configuration file. To make pytest ignore trailing whitespaces and
ignore lengthy exception stack traces you can just write:

.. code-block:: ini

    [pytest]
    doctest_optionflags= NORMALIZE_WHITESPACE IGNORE_EXCEPTION_DETAIL

py.test also introduces new options to allow doctests to run in Python 2 and
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


