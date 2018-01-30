Configuration
=============

Command line options and configuration file settings
-----------------------------------------------------------------

You can get help on command line options and values in INI-style
configurations files by using the general help option::

    pytest -h   # prints options _and_ config file settings

This will display command line and configuration file settings
which were registered by installed plugins.

.. _rootdir:
.. _inifiles:

Initialization: determining rootdir and inifile
-----------------------------------------------

.. versionadded:: 2.7

pytest determines a ``rootdir`` for each test run which depends on
the command line arguments (specified test files, paths) and on
the existence of *ini-files*.  The determined ``rootdir`` and *ini-file* are
printed as part of the pytest header during startup.

Here's a summary what ``pytest`` uses ``rootdir`` for:

* Construct *nodeids* during collection; each test is assigned
  a unique *nodeid* which is rooted at the ``rootdir`` and takes in account full path,
  class name, function name and parametrization (if any).

* Is used by plugins as a stable location to store project/test run specific information;
  for example, the internal :ref:`cache <cache>` plugin creates a ``.cache`` subdirectory
  in ``rootdir`` to store its cross-test run state.

Important to emphasize that ``rootdir`` is **NOT** used to modify ``sys.path``/``PYTHONPATH`` or
influence how modules are imported. See :ref:`pythonpath` for more details.

Finding the ``rootdir``
~~~~~~~~~~~~~~~~~~~~~~~

Here is the algorithm which finds the rootdir from ``args``:

- determine the common ancestor directory for the specified ``args`` that are
  recognised as paths that exist in the file system. If no such paths are
  found, the common ancestor directory is set to the current working directory.

- look for ``pytest.ini``, ``tox.ini`` and ``setup.cfg`` files in the ancestor
  directory and upwards.  If one is matched, it becomes the ini-file and its
  directory becomes the rootdir.

- if no ini-file was found, look for ``setup.py`` upwards from the common
  ancestor directory to determine the ``rootdir``.

- if no ``setup.py`` was found, look for ``pytest.ini``, ``tox.ini`` and
  ``setup.cfg`` in each of the specified ``args`` and upwards. If one is
  matched, it becomes the ini-file and its directory becomes the rootdir.

- if no ini-file was found, use the already determined common ancestor as root
  directory. This allows the use of pytest in structures that are not part of
  a package and don't have any particular ini-file configuration.

If no ``args`` are given, pytest collects test below the current working
directory and also starts determining the rootdir from there.

:warning: custom pytest plugin commandline arguments may include a path, as in
    ``pytest --log-output ../../test.log args``. Then ``args`` is mandatory,
    otherwise pytest uses the folder of test.log for rootdir determination
    (see also `issue 1435 <https://github.com/pytest-dev/pytest/issues/1435>`_).
    A dot ``.`` for referencing to the current working directory is also
    possible.

Note that an existing ``pytest.ini`` file will always be considered a match,
whereas ``tox.ini`` and ``setup.cfg`` will only match if they contain a
``[pytest]`` or ``[tool:pytest]`` section, respectively. Options from multiple ini-files candidates are never
merged - the first one wins (``pytest.ini`` always wins, even if it does not
contain a ``[pytest]`` section).

The ``config`` object will subsequently carry these attributes:

- ``config.rootdir``: the determined root directory, guaranteed to exist.

- ``config.inifile``: the determined ini-file, may be ``None``.

The rootdir is used a reference directory for constructing test
addresses ("nodeids") and can be used also by plugins for storing
per-testrun information.

Example::

    pytest path/to/testdir path/other/

will determine the common ancestor as ``path`` and then
check for ini-files as follows::

    # first look for pytest.ini files
    path/pytest.ini
    path/setup.cfg  # must also contain [tool:pytest] section to match
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
    addopts = -ra -q

Alternatively, you can set a ``PYTEST_ADDOPTS`` environment variable to add command
line options while the environment is in use::

    export PYTEST_ADDOPTS="-v"

Here's how the command-line is built in the presence of ``addopts`` or the environment variable::

    <pytest.ini:addopts> $PYTEST_ADDOTPS <extra command-line arguments>

So if the user executes in the command-line::

    pytest -m slow

The actual command line executed is::

    pytest -ra -q -v -m slow

Note that as usual for other command-line applications, in case of conflicting options the last one wins, so the example
above will show verbose output because ``-v`` overwrites ``-q``.


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

   issuing ``pytest test_hello.py`` actually means::

        pytest --maxfail=2 -rf test_hello.py

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

   Default patterns are ``'.*', 'build', 'dist', 'CVS', '_darcs', '{arch}', '*.egg', 'venv'``.
   Setting a ``norecursedirs`` replaces the default.  Here is an example of
   how to avoid certain directories:

   .. code-block:: ini

        # content of pytest.ini
        [pytest]
        norecursedirs = .svn _build tmp*

   This would tell ``pytest`` to not look into typical subversion or
   sphinx-build directories or into any ``tmp`` prefixed directory.  
   
   Additionally, ``pytest`` will attempt to intelligently identify and ignore a
   virtualenv by the presence of an activation script.  Any directory deemed to
   be the root of a virtual environment will not be considered during test
   collection unless ``‑‑collect‑in‑virtualenv`` is given.  Note also that
   ``norecursedirs`` takes precedence over ``‑‑collect‑in‑virtualenv``; e.g. if
   you intend to run tests in a virtualenv with a base directory that matches
   ``'.*'`` you *must* override ``norecursedirs`` in addition to using the
   ``‑‑collect‑in‑virtualenv`` flag.

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
   are considered as test modules. By default, pytest will consider
   any file matching with ``test_*.py`` and ``*_test.py`` globs as a test
   module.

.. confval:: python_classes

   One or more name prefixes or glob-style patterns determining which classes
   are considered for test collection. By default, pytest will consider any
   class prefixed with ``Test`` as a test collection.  Here is an example of how
   to collect tests from classes that end in ``Suite``:

   .. code-block:: ini

        # content of pytest.ini
        [pytest]
        python_classes = *Suite

   Note that ``unittest.TestCase`` derived classes are always collected
   regardless of this option, as ``unittest``'s own collection framework is used
   to collect those tests.

.. confval:: python_functions

   One or more name prefixes or glob-patterns determining which test functions
   and methods are considered tests. By default, pytest will consider any
   function prefixed with ``test`` as a test.  Here is an example of how
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
   :doc:`See how pytest handles doctests <doctest>`.

.. confval:: confcutdir

   Sets a directory where search upwards for ``conftest.py`` files stops.
   By default, pytest will stop searching for ``conftest.py`` files upwards
   from ``pytest.ini``/``tox.ini``/``setup.cfg`` of the project if any,
   or up to the file-system root.


.. confval:: filterwarnings

   .. versionadded:: 3.1

   Sets a list of filters and actions that should be taken for matched
   warnings. By default all warnings emitted during the test session
   will be displayed in a summary at the end of the test session.

   .. code-block:: ini

        # content of pytest.ini
        [pytest]
        filterwarnings =
            error
            ignore::DeprecationWarning

   This tells pytest to ignore deprecation warnings and turn all other warnings
   into errors. For more information please refer to :ref:`warnings`.

.. confval:: cache_dir

   .. versionadded:: 3.2

   Sets a directory where stores content of cache plugin. Default directory is
   ``.cache`` which is created in :ref:`rootdir <rootdir>`. Directory may be
   relative or absolute path. If setting relative path, then directory is created
   relative to :ref:`rootdir <rootdir>`. Additionally path may contain environment
   variables, that will be expanded. For more information about cache plugin
   please refer to :ref:`cache_provider`.


.. confval:: console_output_style

   .. versionadded:: 3.3

   Sets the console output style while running tests:

   * ``classic``: classic pytest output.
   * ``progress``: like classic pytest output, but with a progress indicator.

   The default is ``progress``, but you can fallback to ``classic`` if you prefer or
   the new mode is causing unexpected problems:

   .. code-block:: ini

        # content of pytest.ini
        [pytest]
        console_output_style = classic
