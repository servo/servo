Basic test configuration
===================================

Command line options and configuration file settings
-----------------------------------------------------------------

You can get help on command line options and values in INI-style
configurations files by using the general help option::

    py.test -h   # prints options _and_ config file settings

This will display command line and configuration file settings
which were registered by installed plugins.

.. _rootdir:
.. _inifiles:

initialization: determining rootdir and inifile
-----------------------------------------------

.. versionadded:: 2.7

pytest determines a "rootdir" for each test run which depends on
the command line arguments (specified test files, paths) and on
the existence of inifiles.  The determined rootdir and ini-file are
printed as part of the pytest header.  The rootdir is used for constructing
"nodeids" during collection and may also be used by plugins to store
project/testrun-specific information.

Here is the algorithm which finds the rootdir from ``args``:

- determine the common ancestor directory for the specified ``args``.

- look for ``pytest.ini``, ``tox.ini`` and ``setup.cfg`` files in the
  ancestor directory and upwards.  If one is matched, it becomes the
  ini-file and its directory becomes the rootdir.  An existing
  ``pytest.ini`` file will always be considered a match whereas
  ``tox.ini`` and ``setup.cfg`` will only match if they contain
  a ``[pytest]`` section.

- if no ini-file was found, look for ``setup.py`` upwards from
  the common ancestor directory to determine the ``rootdir``.

- if no ini-file and no ``setup.py`` was found, use the already
  determined common ancestor as root directory.  This allows to
  work with pytest in structures that are not part of a package
  and don't have any particular ini-file configuration.

Note that options from multiple ini-files candidates are never merged,
the first one wins (``pytest.ini`` always wins even if it does not
contain a ``[pytest]`` section).

The ``config`` object will subsequently carry these attributes:

- ``config.rootdir``: the determined root directory, guaranteed to exist.

- ``config.inifile``: the determined ini-file, may be ``None``.

The rootdir is used a reference directory for constructing test
addresses ("nodeids") and can be used also by plugins for storing
per-testrun information.

Example::

    py.test path/to/testdir path/other/

will determine the common ancestor as ``path`` and then
check for ini-files as follows::

    # first look for pytest.ini files
    path/pytest.ini
    path/setup.cfg  # must also contain [pytest] section to match
    path/tox.ini    # must also contain [pytest] section to match
    pytest.ini
    ... # all the way down to the root

    # now look for setup.py
    path/setup.py
    setup.py
    ... # all the way down to the root


.. _`how to change command line options defaults`:
.. _`adding default options`:

How to change command line options defaults
------------------------------------------------

It can be tedious to type the same series of command line options
every time you use ``pytest``.  For example, if you always want to see
detailed info on skipped and xfailed tests, as well as have terser "dot"
progress output, you can write it into a configuration file:

.. code-block:: ini

    # content of pytest.ini
    # (or tox.ini or setup.cfg)
    [pytest]
    addopts = -rsxX -q

Alternatively, you can set a PYTEST_ADDOPTS environment variable to add command
line options while the environment is in use::

    export PYTEST_ADDOPTS="-rsxX -q"

From now on, running ``pytest`` will add the specified options.



Builtin configuration file options
----------------------------------------------

.. confval:: minversion

   Specifies a minimal pytest version required for running tests.

        minversion = 2.1  # will fail if we run with pytest-2.0

.. confval:: addopts

   Add the specified ``OPTS`` to the set of command line arguments as if they
   had been specified by the user. Example: if you have this ini file content:

   .. code-block:: ini

        [pytest]
        addopts = --maxfail=2 -rf  # exit after 2 failures, report fail info

   issuing ``py.test test_hello.py`` actually means::

        py.test --maxfail=2 -rf test_hello.py

   Default is to add no options.

.. confval:: norecursedirs

   Set the directory basename patterns to avoid when recursing
   for test discovery.  The individual (fnmatch-style) patterns are
   applied to the basename of a directory to decide if to recurse into it.
   Pattern matching characters::

        *       matches everything
        ?       matches any single character
        [seq]   matches any character in seq
        [!seq]  matches any char not in seq

   Default patterns are ``'.*', 'CVS', '_darcs', '{arch}', '*.egg'``.
   Setting a ``norecursedirs`` replaces the default.  Here is an example of
   how to avoid certain directories:

   .. code-block:: ini

        # content of setup.cfg
        [pytest]
        norecursedirs = .svn _build tmp*

   This would tell ``pytest`` to not look into typical subversion or
   sphinx-build directories or into any ``tmp`` prefixed directory.

.. confval:: testpaths

   .. versionadded:: 2.8

   Sets list of directories that should be searched for tests when
   no specific directories, files or test ids are given in the command line when
   executing pytest from the :ref:`rootdir <rootdir>` directory.
   Useful when all project tests are in a known location to speed up
   test collection and to avoid picking up undesired tests by accident.

   .. code-block:: ini

        # content of pytest.ini
        [pytest]
        testpaths = testing doc

   This tells pytest to only look for tests in ``testing`` and ``doc``
   directories when executing from the root directory.

.. confval:: python_files

   One or more Glob-style file patterns determining which python files
   are considered as test modules.

.. confval:: python_classes

   One or more name prefixes or glob-style patterns determining which classes
   are considered for test collection. Here is an example of how to collect
   tests from classes that end in ``Suite``:

   .. code-block:: ini

        # content of pytest.ini
        [pytest]
        python_classes = *Suite

   Note that ``unittest.TestCase`` derived classes are always collected
   regardless of this option, as ``unittest``'s own collection framework is used
   to collect those tests.

.. confval:: python_functions

   One or more name prefixes or glob-patterns determining which test functions
   and methods are considered tests. Here is an example of how
   to collect test functions and methods that end in ``_test``:

   .. code-block:: ini

        # content of pytest.ini
        [pytest]
        python_functions = *_test

   Note that this has no effect on methods that live on a ``unittest
   .TestCase`` derived class, as ``unittest``'s own collection framework is used
   to collect those tests.

   See :ref:`change naming conventions` for more detailed examples.

.. confval:: doctest_optionflags

   One or more doctest flag names from the standard ``doctest`` module.
   :doc:`See how py.test handles doctests <doctest>`.

.. confval:: confcutdir

   Sets a directory where search upwards for ``conftest.py`` files stops.
   By default, pytest will stop searching for ``conftest.py`` files upwards
   from ``pytest.ini``/``tox.ini``/``setup.cfg`` of the project if any,
   or up to the file-system root.
