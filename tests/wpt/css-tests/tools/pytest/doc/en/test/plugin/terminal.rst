
Implements terminal reporting of the full testing process.
==========================================================


.. contents::
  :local:

This is a good source for looking at the various reporting hooks.

command line options
--------------------


``-v, --verbose``
    increase verbosity.
``-r chars``
    show extra test summary info as specified by chars (f)ailed, (s)skipped, (x)failed, (X)passed.
``-l, --showlocals``
    show locals in tracebacks (disabled by default).
``--report=opts``
    (deprecated, use -r)
``--tb=style``
    traceback print mode (long/short/line/no).
``--fulltrace``
    don't cut any tracebacks (default is to cut).
``--fixtures``
    show available function arguments, sorted by plugin

Start improving this plugin in 30 seconds
=========================================


1. Download `pytest_terminal.py`_ plugin source code
2. put it somewhere as ``pytest_terminal.py`` into your import path
3. a subsequent ``pytest`` run will use your local version

Checkout customize_, other plugins_ or `get in contact`_.

.. include:: links.txt
