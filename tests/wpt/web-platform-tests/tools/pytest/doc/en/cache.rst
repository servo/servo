Cache: working with cross-testrun state
=======================================

.. versionadded:: 2.8

.. warning::

  The functionality of this core plugin was previously distributed
  as a third party plugin named ``pytest-cache``.  The core plugin
  is compatible regarding command line options and API usage except that you
  can only store/receive data between test runs that is json-serializable.


Usage
---------

The plugin provides two command line options to rerun failures from the
last ``py.test`` invocation:

* ``--lf``, ``--last-failed`` - to only re-run the failures.
* ``--ff``, ``--failed-first`` - to run the failures first and then the rest of
  the tests.

For cleanup (usually not needed), a ``--cache-clear`` option allows to remove
all cross-session cache contents ahead of a test run.

Other plugins may access the `config.cache`_ object to set/get 
**json encodable** values between ``py.test`` invocations.

.. note::

    This plugin is enabled by default, but can be disabled if needed: see
    :ref:`cmdunregister` (the internal name for this plugin is
    ``cacheprovider``).


Rerunning only failures or failures first
-----------------------------------------------

First, let's create 50 test invocation of which only 2 fail::

    # content of test_50.py
    import pytest

    @pytest.mark.parametrize("i", range(50))
    def test_num(i):
        if i in (17, 25):
           pytest.fail("bad luck")

If you run this for the first time you will see two failures::

    $ py.test -q
    .................F.......F........................
    ======= FAILURES ========
    _______ test_num[17] ________
    
    i = 17
    
        @pytest.mark.parametrize("i", range(50))
        def test_num(i):
            if i in (17, 25):
    >          pytest.fail("bad luck")
    E          Failed: bad luck
    
    test_50.py:6: Failed
    _______ test_num[25] ________
    
    i = 25
    
        @pytest.mark.parametrize("i", range(50))
        def test_num(i):
            if i in (17, 25):
    >          pytest.fail("bad luck")
    E          Failed: bad luck
    
    test_50.py:6: Failed
    2 failed, 48 passed in 0.12 seconds

If you then run it with ``--lf``::

    $ py.test --lf
    ======= test session starts ========
    platform linux -- Python 3.4.0, pytest-2.9.1, py-1.4.31, pluggy-0.3.1
    run-last-failure: rerun last 2 failures
    rootdir: $REGENDOC_TMPDIR, inifile: 
    collected 50 items
    
    test_50.py FF
    
    ======= FAILURES ========
    _______ test_num[17] ________
    
    i = 17
    
        @pytest.mark.parametrize("i", range(50))
        def test_num(i):
            if i in (17, 25):
    >          pytest.fail("bad luck")
    E          Failed: bad luck
    
    test_50.py:6: Failed
    _______ test_num[25] ________
    
    i = 25
    
        @pytest.mark.parametrize("i", range(50))
        def test_num(i):
            if i in (17, 25):
    >          pytest.fail("bad luck")
    E          Failed: bad luck
    
    test_50.py:6: Failed
    ======= 2 failed, 48 deselected in 0.12 seconds ========

You have run only the two failing test from the last run, while 48 tests have
not been run ("deselected").

Now, if you run with the ``--ff`` option, all tests will be run but the first
previous failures will be executed first (as can be seen from the series
of ``FF`` and dots)::

    $ py.test --ff
    ======= test session starts ========
    platform linux -- Python 3.4.0, pytest-2.9.1, py-1.4.31, pluggy-0.3.1
    run-last-failure: rerun last 2 failures first
    rootdir: $REGENDOC_TMPDIR, inifile: 
    collected 50 items
    
    test_50.py FF................................................
    
    ======= FAILURES ========
    _______ test_num[17] ________
    
    i = 17
    
        @pytest.mark.parametrize("i", range(50))
        def test_num(i):
            if i in (17, 25):
    >          pytest.fail("bad luck")
    E          Failed: bad luck
    
    test_50.py:6: Failed
    _______ test_num[25] ________
    
    i = 25
    
        @pytest.mark.parametrize("i", range(50))
        def test_num(i):
            if i in (17, 25):
    >          pytest.fail("bad luck")
    E          Failed: bad luck
    
    test_50.py:6: Failed
    ======= 2 failed, 48 passed in 0.12 seconds ========

.. _`config.cache`:

The new config.cache object
--------------------------------

.. regendoc:wipe

Plugins or conftest.py support code can get a cached value using the
pytest ``config`` object.  Here is a basic example plugin which
implements a :ref:`fixture` which re-uses previously created state
across py.test invocations::

    # content of test_caching.py
    import pytest
    import time

    @pytest.fixture
    def mydata(request):
        val = request.config.cache.get("example/value", None)
        if val is None:
            time.sleep(9*0.6) # expensive computation :)
            val = 42
            request.config.cache.set("example/value", val)
        return val

    def test_function(mydata):
        assert mydata == 23

If you run this command once, it will take a while because
of the sleep::

    $ py.test -q
    F
    ======= FAILURES ========
    _______ test_function ________
    
    mydata = 42
    
        def test_function(mydata):
    >       assert mydata == 23
    E       assert 42 == 23
    
    test_caching.py:14: AssertionError
    1 failed in 0.12 seconds

If you run it a second time the value will be retrieved from
the cache and this will be quick::

    $ py.test -q
    F
    ======= FAILURES ========
    _______ test_function ________
    
    mydata = 42
    
        def test_function(mydata):
    >       assert mydata == 23
    E       assert 42 == 23
    
    test_caching.py:14: AssertionError
    1 failed in 0.12 seconds

See the `cache-api`_ for more details.


Inspecting Cache content
-------------------------------

You can always peek at the content of the cache using the
``--cache-clear`` command line option::

    $ py.test --cache-clear
    ======= test session starts ========
    platform linux -- Python 3.4.0, pytest-2.9.1, py-1.4.31, pluggy-0.3.1
    rootdir: $REGENDOC_TMPDIR, inifile: 
    collected 1 items
    
    test_caching.py F
    
    ======= FAILURES ========
    _______ test_function ________
    
    mydata = 42
    
        def test_function(mydata):
    >       assert mydata == 23
    E       assert 42 == 23
    
    test_caching.py:14: AssertionError
    ======= 1 failed in 0.12 seconds ========

Clearing Cache content
-------------------------------

You can instruct pytest to clear all cache files and values
by adding the ``--cache-clear`` option like this::

    py.test --cache-clear

This is recommended for invocations from Continous Integration
servers where isolation and correctness is more important
than speed.


.. _`cache-api`:

config.cache API
------------------

The ``config.cache`` object allows other plugins,
including ``conftest.py`` files,
to safely and flexibly store and retrieve values across
test runs because the ``config`` object is available
in many places.

Under the hood, the cache plugin uses the simple
dumps/loads API of the json stdlib module

.. currentmodule:: _pytest.cacheprovider

.. automethod:: Cache.get
.. automethod:: Cache.set
.. automethod:: Cache.makedir
