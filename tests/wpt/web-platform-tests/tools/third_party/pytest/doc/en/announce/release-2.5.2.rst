pytest-2.5.2: fixes
===========================================================================

pytest is a mature Python testing tool with more than a 1000 tests
against itself, passing on many different interpreters and platforms.

The 2.5.2 release fixes a few bugs with two maybe-bugs remaining and
actively being worked on (and waiting for the bug reporter's input).
We also have a new contribution guide thanks to Piotr Banaszkiewicz
and others.

See docs at:

    http://pytest.org

As usual, you can upgrade from pypi via::

    pip install -U pytest

Thanks to the following people who contributed to this release:

    Anatoly Bubenkov
    Ronny Pfannschmidt
    Floris Bruynooghe
    Bruno Oliveira
    Andreas Pelme
    Jurko Gospodnetić
    Piotr Banaszkiewicz
    Simon Liedtke
    lakka
    Lukasz Balcerzak
    Philippe Muller
    Daniel Hahler

have fun,
holger krekel

2.5.2
-----------------------------------

- fix issue409 -- better interoperate with cx_freeze by not
  trying to import from collections.abc which causes problems
  for py27/cx_freeze.  Thanks Wolfgang L. for reporting and tracking it down.

- fixed docs and code to use "pytest" instead of "py.test" almost everywhere.
  Thanks Jurko Gospodnetic for the complete PR.

- fix issue425: mention at end of "py.test -h" that --markers
  and --fixtures work according to specified test path (or current dir)

- fix issue413: exceptions with unicode attributes are now printed
  correctly also on python2 and with pytest-xdist runs. (the fix
  requires py-1.4.20)

- copy, cleanup and integrate py.io capture
  from pylib 1.4.20.dev2 (rev 13d9af95547e)

- address issue416: clarify docs as to conftest.py loading semantics

- fix issue429: comparing byte strings with non-ascii chars in assert
  expressions now work better.  Thanks Floris Bruynooghe.

- make capfd/capsys.capture private, its unused and shouldn't be exposed
