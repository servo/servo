===============
Version History
===============

.. automodule:: more_itertools

4.2.0
-----

* New itertools:
    * :func:`map_reduce` (thanks to pylang)
    * :func:`prepend` (from the `Python 3.7 docs <https://docs.python.org/3.7/library/itertools.html#itertools-recipes>`_)

* Improvements to existing itertools:
    * :func:`bucket` now complies with PEP 479 (thanks to irmen)

* Other changes:
   * Python 3.7 is now supported (thanks to irmen)
   * Python 3.3 is no longer supported
   * The test suite no longer requires third-party modules to run
   * The API docs now include links to source code

4.1.0
-----

* New itertools:
    * :func:`split_at` (thanks to michael-celani)
    * :func:`circular_shifts` (thanks to hiqua)
    * :func:`make_decorator` - see the blog post `Yo, I heard you like decorators <https://sites.google.com/site/bbayles/index/decorator_factory>`_
      for a tour (thanks to pylang)
    * :func:`always_reversible` (thanks to michael-celani)
    * :func:`nth_combination` (from the `Python 3.7 docs <https://docs.python.org/3.7/library/itertools.html#itertools-recipes>`_)

* Improvements to existing itertools:
    * :func:`seekable` now has an ``elements`` method to return cached items.
    * The performance tradeoffs between :func:`roundrobin` and
      :func:`interleave_longest` are now documented (thanks michael-celani,
      pylang, and MSeifert04)

4.0.1
-----

* No code changes - this release fixes how the docs display on PyPI.

4.0.0
-----

* New itertools:
    * :func:`consecutive_groups` (Based on the example in the `Python 2.4 docs <https://docs.python.org/release/2.4.4/lib/itertools-example.html>`_)
    * :func:`seekable` (If you're looking for how to "reset" an iterator,
      you're in luck!)
    * :func:`exactly_n` (thanks to michael-celani)
    * :func:`run_length.encode` and :func:`run_length.decode`
    * :func:`difference`

* Improvements to existing itertools:
    * The number of items between filler elements in :func:`intersperse` can
      now be specified (thanks to pylang)
    * :func:`distinct_permutations` and :func:`peekable` got some minor
      adjustments (thanks to MSeifert04)
    * :func:`always_iterable` now returns an iterator object. It also now
      allows different types to be considered iterable (thanks to jaraco)
    * :func:`bucket` can now limit the keys it stores in memory
    * :func:`one` now allows for custom exceptions (thanks to kalekundert)

* Other changes:
    * A few typos were fixed (thanks to EdwardBetts)
    * All tests can now be run with ``python setup.py test``

The major version update is due to the change in the return value of :func:`always_iterable`.
It now always returns iterator objects:

.. code-block:: python

    >>> from more_itertools import always_iterable
    # Non-iterable objects are wrapped with iter(tuple(obj))
    >>> always_iterable(12345)
    <tuple_iterator object at 0x7fb24c9488d0>
    >>> list(always_iterable(12345))
    [12345]
    # Iterable objects are wrapped with iter()
    >>> always_iterable([1, 2, 3, 4, 5])
    <list_iterator object at 0x7fb24c948c50>

3.2.0
-----

* New itertools:
    * :func:`lstrip`, :func:`rstrip`, and :func:`strip`
      (thanks to MSeifert04 and pylang)
    * :func:`islice_extended`
* Improvements to existing itertools:
    * Some bugs with slicing :func:`peekable`-wrapped iterables were fixed

3.1.0
-----

* New itertools:
    * :func:`numeric_range` (Thanks to BebeSparkelSparkel and MSeifert04)
    * :func:`count_cycle` (Thanks to BebeSparkelSparkel)
    * :func:`locate` (Thanks to pylang and MSeifert04)
* Improvements to existing itertools:
    * A few itertools are now slightly faster due to some function
      optimizations. (Thanks to MSeifert04)
* The docs have been substantially revised with installation notes,
  categories for library functions, links, and more. (Thanks to pylang)


3.0.0
-----

* Removed itertools:
    * ``context`` has been removed due to a design flaw - see below for
      replacement options. (thanks to NeilGirdhar)
* Improvements to existing itertools:
    * ``side_effect`` now supports ``before`` and ``after`` keyword
      arguments. (Thanks to yardsale8)
* PyPy and PyPy3 are now supported.

The major version change is due to the removal of the ``context`` function.
Replace it with standard ``with`` statement context management:

.. code-block:: python

    # Don't use context() anymore
    file_obj = StringIO()
    consume(print(x, file=f) for f in context(file_obj) for x in u'123')

    # Use a with statement instead
    file_obj = StringIO()
    with file_obj as f:
        consume(print(x, file=f) for x in u'123')

2.6.0
-----

* New itertools:
    * ``adjacent`` and ``groupby_transform`` (Thanks to diazona)
    * ``always_iterable`` (Thanks to jaraco)
    * (Removed in 3.0.0) ``context`` (Thanks to yardsale8)
    * ``divide`` (Thanks to mozbhearsum)
* Improvements to existing itertools:
    * ``ilen`` is now slightly faster. (Thanks to wbolster)
    * ``peekable`` can now prepend items to an iterable. (Thanks to diazona)

2.5.0
-----

* New itertools:
    * ``distribute`` (Thanks to mozbhearsum and coady)
    * ``sort_together`` (Thanks to clintval)
    * ``stagger`` and ``zip_offset`` (Thanks to joshbode)
    * ``padded``
* Improvements to existing itertools:
    * ``peekable`` now handles negative indexes and slices with negative
      components properly.
    * ``intersperse`` is now slightly faster. (Thanks to pylang)
    * ``windowed`` now accepts a ``step`` keyword argument.
      (Thanks to pylang)
* Python 3.6 is now supported.

2.4.1
-----

* Move docs 100% to readthedocs.io.

2.4
-----

* New itertools:
    * ``accumulate``, ``all_equal``, ``first_true``, ``partition``, and
      ``tail`` from the itertools documentation.
    * ``bucket`` (Thanks to Rosuav and cvrebert)
    * ``collapse`` (Thanks to abarnet)
    * ``interleave`` and ``interleave_longest`` (Thanks to abarnet)
    * ``side_effect`` (Thanks to nvie)
    * ``sliced`` (Thanks to j4mie and coady)
    * ``split_before`` and ``split_after`` (Thanks to astronouth7303)
    * ``spy`` (Thanks to themiurgo and mathieulongtin)
* Improvements to existing itertools:
    * ``chunked`` is now simpler and more friendly to garbage collection.
      (Contributed by coady, with thanks to piskvorky)
    * ``collate`` now delegates to ``heapq.merge`` when possible.
      (Thanks to kmike and julianpistorius)
    * ``peekable``-wrapped iterables are now indexable and sliceable.
      Iterating through ``peekable``-wrapped iterables is also faster.
    * ``one`` and ``unique_to_each`` have been simplified.
      (Thanks to coady)


2.3
-----

* Added ``one`` from ``jaraco.util.itertools``. (Thanks, jaraco!)
* Added ``distinct_permutations`` and ``unique_to_each``. (Contributed by
  bbayles)
* Added ``windowed``. (Contributed by bbayles, with thanks to buchanae,
  jaraco, and abarnert)
* Simplified the implementation of ``chunked``. (Thanks, nvie!)
* Python 3.5 is now supported. Python 2.6 is no longer supported.
* Python 3 is now supported directly; there is no 2to3 step.

2.2
-----

* Added ``iterate`` and ``with_iter``. (Thanks, abarnert!)

2.1
-----

* Added (tested!) implementations of the recipes from the itertools
  documentation. (Thanks, Chris Lonnen!)
* Added ``ilen``. (Thanks for the inspiration, Matt Basta!)

2.0
-----

* ``chunked`` now returns lists rather than tuples. After all, they're
  homogeneous. This slightly backward-incompatible change is the reason for
  the major version bump.
* Added ``@consumer``.
* Improved test machinery.

1.1
-----

* Added ``first`` function.
* Added Python 3 support.
* Added a default arg to ``peekable.peek()``.
* Noted how to easily test whether a peekable iterator is exhausted.
* Rewrote documentation.

1.0
-----

* Initial release, with ``collate``, ``peekable``, and ``chunked``. Could
  really use better docs.
