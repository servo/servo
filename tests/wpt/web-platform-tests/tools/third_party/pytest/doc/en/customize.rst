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

``--rootdir=path`` command-line option can be used to force a specific directory.
The directory passed may contain environment variables when it is used in conjunction
with ``addopts`` in a ``pytest.ini`` file.

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

For the full list of options consult the :ref:`reference documentation <ini options ref>`.
