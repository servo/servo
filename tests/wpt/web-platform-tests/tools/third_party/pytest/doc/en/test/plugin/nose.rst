
nose-compatibility plugin: allow to run nose test suites natively.
==================================================================


.. contents::
  :local:

This is an experimental plugin for allowing to run tests written
in 'nosetests' style with ``pytest``.

Usage
-------------

type::

    pytest  # instead of 'nosetests'

and you should be able to run nose style tests and at the same
time can make full use of pytest's capabilities.

Supported nose Idioms
----------------------

* setup and teardown at module/class/method level
* SkipTest exceptions and markers
* setup/teardown decorators
* yield-based tests and their setup
* general usage of nose utilities

Unsupported idioms / issues
----------------------------------

- nose-style doctests are not collected and executed correctly,
  also fixtures don't work.

- no nose-configuration is recognized

If you find other issues or have suggestions please run::

    pytest --pastebin=all

and send the resulting URL to a ``pytest`` contact channel,
at best to the mailing list.

Start improving this plugin in 30 seconds
=========================================


1. Download `pytest_nose.py`_ plugin source code
2. put it somewhere as ``pytest_nose.py`` into your import path
3. a subsequent ``pytest`` run will use your local version

Checkout customize_, other plugins_ or `get in contact`_.

.. include:: links.txt
