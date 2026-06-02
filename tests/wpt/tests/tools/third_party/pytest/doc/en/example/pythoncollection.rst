Changing standard (Python) test discovery
===============================================

Ignore paths during test collection
-----------------------------------

You can easily ignore certain test directories and modules during collection
by passing the ``--ignore=path`` option on the cli. ``pytest`` allows multiple
``--ignore`` options. Example:

.. code-block:: text

    tests/
    |-- example
    |   |-- test_example_01.py
    |   |-- test_example_02.py
    |   '-- test_example_03.py
    |-- foobar
    |   |-- test_foobar_01.py
    |   |-- test_foobar_02.py
    |   '-- test_foobar_03.py
    '-- hello
        '-- world
            |-- test_world_01.py
            |-- test_world_02.py
            '-- test_world_03.py

Now if you invoke ``pytest`` with ``--ignore=tests/foobar/test_foobar_03.py --ignore=tests/hello/``,
you will see that ``pytest`` only collects test-modules, which do not match the patterns specified:

.. code-block:: pytest

    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-5.x.y, py-1.x.y, pluggy-0.x.y
    rootdir: $REGENDOC_TMPDIR, inifile:
    collected 5 items

    tests/example/test_example_01.py .                                   [ 20%]
    tests/example/test_example_02.py .                                   [ 40%]
    tests/example/test_example_03.py .                                   [ 60%]
    tests/foobar/test_foobar_01.py .                                     [ 80%]
    tests/foobar/test_foobar_02.py .                                     [100%]

    ========================= 5 passed in 0.02 seconds =========================

The ``--ignore-glob`` option allows to ignore test file paths based on Unix shell-style wildcards.
If you want to exclude test-modules that end with ``_01.py``, execute ``pytest`` with ``--ignore-glob='*_01.py'``.

Deselect tests during test collection
-------------------------------------

Tests can individually be deselected during collection by passing the ``--deselect=item`` option.
For example, say ``tests/foobar/test_foobar_01.py`` contains ``test_a`` and ``test_b``.
You can run all of the tests within ``tests/`` *except* for ``tests/foobar/test_foobar_01.py::test_a``
by invoking ``pytest`` with ``--deselect tests/foobar/test_foobar_01.py::test_a``.
``pytest`` allows multiple ``--deselect`` options.

Keeping duplicate paths specified from command line
----------------------------------------------------

Default behavior of ``pytest`` is to ignore duplicate paths specified from the command line.
Example:

.. code-block:: pytest

    pytest path_a path_a

    ...
    collected 1 item
    ...

Just collect tests once.

To collect duplicate tests, use the ``--keep-duplicates`` option on the cli.
Example:

.. code-block:: pytest

    pytest --keep-duplicates path_a path_a

    ...
    collected 2 items
    ...

As the collector just works on directories, if you specify twice a single test file, ``pytest`` will
still collect it twice, no matter if the ``--keep-duplicates`` is not specified.
Example:

.. code-block:: pytest

    pytest test_a.py test_a.py

    ...
    collected 2 items
    ...


Changing directory recursion
-----------------------------------------------------

You can set the :confval:`norecursedirs` option in an ini-file, for example your ``pytest.ini`` in the project root directory:

.. code-block:: ini

    # content of pytest.ini
    [pytest]
    norecursedirs = .svn _build tmp*

This would tell ``pytest`` to not recurse into typical subversion or sphinx-build directories or into any ``tmp`` prefixed directory.

.. _`change naming conventions`:

Changing naming conventions
-----------------------------------------------------

You can configure different naming conventions by setting
the :confval:`python_files`, :confval:`python_classes` and
:confval:`python_functions` in your :ref:`configuration file <config file formats>`.
Here is an example:

.. code-block:: ini

    # content of pytest.ini
    # Example 1: have pytest look for "check" instead of "test"
    [pytest]
    python_files = check_*.py
    python_classes = Check
    python_functions = *_check

This would make ``pytest`` look for tests in files that match the ``check_*
.py`` glob-pattern, ``Check`` prefixes in classes, and functions and methods
that match ``*_check``. For example, if we have:

.. code-block:: python

    # content of check_myapp.py
    class CheckMyApp:
        def simple_check(self):
            pass

        def complex_check(self):
            pass

The test collection would look like this:

.. code-block:: pytest

    $ pytest --collect-only
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    configfile: pytest.ini
    collected 2 items

    <Dir pythoncollection.rst-200>
      <Module check_myapp.py>
        <Class CheckMyApp>
          <Function simple_check>
          <Function complex_check>

    ======================== 2 tests collected in 0.12s ========================

You can check for multiple glob patterns by adding a space between the patterns:

.. code-block:: ini

    # Example 2: have pytest look for files with "test" and "example"
    # content of pytest.ini
    [pytest]
    python_files = test_*.py example_*.py

.. note::

   the ``python_functions`` and ``python_classes`` options has no effect
   for ``unittest.TestCase`` test discovery because pytest delegates
   discovery of test case methods to unittest code.

Interpreting cmdline arguments as Python packages
-----------------------------------------------------

You can use the ``--pyargs`` option to make ``pytest`` try
interpreting arguments as python package names, deriving
their file system path and then running the test. For
example if you have unittest2 installed you can type:

.. code-block:: bash

    pytest --pyargs unittest2.test.test_skipping -q

which would run the respective test module.  Like with
other options, through an ini-file and the :confval:`addopts` option you
can make this change more permanently:

.. code-block:: ini

    # content of pytest.ini
    [pytest]
    addopts = --pyargs

Now a simple invocation of ``pytest NAME`` will check
if NAME exists as an importable package/module and otherwise
treat it as a filesystem path.

Finding out what is collected
-----------------------------------------------

You can always peek at the collection tree without running tests like this:

.. code-block:: pytest

    . $ pytest --collect-only pythoncollection.py
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    configfile: pytest.ini
    collected 3 items

    <Dir pythoncollection.rst-200>
      <Dir CWD>
        <Module pythoncollection.py>
          <Function test_function>
          <Class TestClass>
            <Function test_method>
            <Function test_anothermethod>

    ======================== 3 tests collected in 0.12s ========================

.. _customizing-test-collection:

Customizing test collection
---------------------------

.. regendoc:wipe

You can easily instruct ``pytest`` to discover tests from every Python file:

.. code-block:: ini

    # content of pytest.ini
    [pytest]
    python_files = *.py

However, many projects will have a ``setup.py`` which they don't want to be
imported. Moreover, there may files only importable by a specific python
version. For such cases you can dynamically define files to be ignored by
listing them in a ``conftest.py`` file:

.. code-block:: python

    # content of conftest.py
    import sys

    collect_ignore = ["setup.py"]
    if sys.version_info[0] > 2:
        collect_ignore.append("pkg/module_py2.py")

and then if you have a module file like this:

.. code-block:: python

    # content of pkg/module_py2.py
    def test_only_on_python2():
        try:
            assert 0
        except Exception, e:
            pass

and a ``setup.py`` dummy file like this:

.. code-block:: python

    # content of setup.py
    0 / 0  # will raise exception if imported

If you run with a Python 2 interpreter then you will find the one test and will
leave out the ``setup.py`` file:

.. code-block:: pytest

    #$ pytest --collect-only
    ====== test session starts ======
    platform linux2 -- Python 2.7.10, pytest-2.9.1, py-1.4.31, pluggy-0.3.1
    rootdir: $REGENDOC_TMPDIR, inifile: pytest.ini
    collected 1 items
    <Module 'pkg/module_py2.py'>
      <Function 'test_only_on_python2'>

    ====== 1 tests found in 0.04 seconds ======

If you run with a Python 3 interpreter both the one test and the ``setup.py``
file will be left out:

.. code-block:: pytest

    $ pytest --collect-only
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    configfile: pytest.ini
    collected 0 items

    ======================= no tests collected in 0.12s ========================

It's also possible to ignore files based on Unix shell-style wildcards by adding
patterns to :globalvar:`collect_ignore_glob`.

The following example ``conftest.py`` ignores the file ``setup.py`` and in
addition all files that end with ``*_py2.py`` when executed with a Python 3
interpreter:

.. code-block:: python

    # content of conftest.py
    import sys

    collect_ignore = ["setup.py"]
    if sys.version_info[0] > 2:
        collect_ignore_glob = ["*_py2.py"]

Since Pytest 2.6, users can prevent pytest from discovering classes that start
with ``Test`` by setting a boolean ``__test__`` attribute to ``False``.

.. code-block:: python

    # Will not be discovered as a test
    class TestClass:
        __test__ = False
