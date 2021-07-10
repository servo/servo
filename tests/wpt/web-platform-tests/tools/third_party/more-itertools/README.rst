==============
More Itertools
==============

.. image:: https://coveralls.io/repos/github/erikrose/more-itertools/badge.svg?branch=master
  :target: https://coveralls.io/github/erikrose/more-itertools?branch=master

Python's ``itertools`` library is a gem - you can compose elegant solutions
for a variety of problems with the functions it provides. In ``more-itertools``
we collect additional building blocks, recipes, and routines for working with
Python iterables.

Getting started
===============

To get started, install the library with `pip <https://pip.pypa.io/en/stable/>`_:

.. code-block:: shell

    pip install more-itertools

The recipes from the `itertools docs <https://docs.python.org/3/library/itertools.html#itertools-recipes>`_
are included in the top-level package:

.. code-block:: python

    >>> from more_itertools import flatten
    >>> iterable = [(0, 1), (2, 3)]
    >>> list(flatten(iterable))
    [0, 1, 2, 3]

Several new recipes are available as well:

.. code-block:: python

    >>> from more_itertools import chunked
    >>> iterable = [0, 1, 2, 3, 4, 5, 6, 7, 8]
    >>> list(chunked(iterable, 3))
    [[0, 1, 2], [3, 4, 5], [6, 7, 8]]

    >>> from more_itertools import spy
    >>> iterable = (x * x for x in range(1, 6))
    >>> head, iterable = spy(iterable, n=3)
    >>> list(head)
    [1, 4, 9]
    >>> list(iterable)
    [1, 4, 9, 16, 25]



For the full listing of functions, see the `API documentation <https://more-itertools.readthedocs.io/en/latest/api.html>`_.

Development
===========

``more-itertools`` is maintained by `@erikrose <https://github.com/erikrose>`_
and `@bbayles <https://github.com/bbayles>`_, with help from `many others <https://github.com/erikrose/more-itertools/graphs/contributors>`_.
If you have a problem or suggestion, please file a bug or pull request in this
repository. Thanks for contributing!
