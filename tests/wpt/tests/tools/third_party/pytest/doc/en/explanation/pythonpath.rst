.. _pythonpath:

pytest import mechanisms and ``sys.path``/``PYTHONPATH``
========================================================

.. _`import-modes`:

Import modes
------------

pytest as a testing framework needs to import test modules and ``conftest.py`` files for execution.

Importing files in Python is a non-trivial processes, so aspects of the
import process can be controlled through the ``--import-mode`` command-line flag, which can assume
these values:

.. _`import-mode-prepend`:

* ``prepend`` (default): the directory path containing each module will be inserted into the *beginning*
  of :py:data:`sys.path` if not already there, and then imported with
  the :func:`importlib.import_module <importlib.import_module>` function.

  It is highly recommended to arrange your test modules as packages by adding ``__init__.py`` files to your directories
  containing tests. This will make the tests part of a proper Python package, allowing pytest to resolve their full
  name (for example ``tests.core.test_core`` for ``test_core.py`` inside the ``tests.core`` package).

  If the test directory tree is not arranged as packages, then each test file needs to have a unique name
  compared to the other test files, otherwise pytest will raise an error if it finds two tests with the same name.

  This is the classic mechanism, dating back from the time Python 2 was still supported.

.. _`import-mode-append`:

* ``append``: the directory containing each module is appended to the end of :py:data:`sys.path` if not already
  there, and imported with :func:`importlib.import_module <importlib.import_module>`.

  This better allows to run test modules against installed versions of a package even if the
  package under test has the same import root. For example:

  ::

        testing/__init__.py
        testing/test_pkg_under_test.py
        pkg_under_test/

  the tests will run against the installed version
  of ``pkg_under_test`` when ``--import-mode=append`` is used whereas
  with ``prepend`` they would pick up the local version. This kind of confusion is why
  we advocate for using :ref:`src-layouts <src-layout>`.

  Same as ``prepend``, requires test module names to be unique when the test directory tree is
  not arranged in packages, because the modules will put in :py:data:`sys.modules` after importing.

.. _`import-mode-importlib`:

* ``importlib``: this mode uses more fine control mechanisms provided by :mod:`importlib` to import test modules, without changing :py:data:`sys.path`.

  Advantages of this mode:

  * pytest will not change :py:data:`sys.path` at all.
  * Test module names do not need to be unique -- pytest will generate a unique name automatically based on the ``rootdir``.

  Disadvantages:

  * Test modules can't import each other.
  * Testing utility modules in the tests directories (for example a ``tests.helpers`` module containing test-related functions/classes)
    are not importable. The recommendation in this case it to place testing utility modules together with the application/library
    code, for example ``app.testing.helpers``.

    Important: by "test utility modules" we mean functions/classes which are imported by
    other tests directly; this does not include fixtures, which should be placed in ``conftest.py`` files, along
    with the test modules, and are discovered automatically by pytest.

  It works like this:

  1. Given a certain module path, for example ``tests/core/test_models.py``, derives a canonical name
     like ``tests.core.test_models`` and tries to import it.

     For non-test modules this will work if they are accessible via :py:data:`sys.path`, so
     for example ``.env/lib/site-packages/app/core.py`` will be importable as ``app.core``.
     This is happens when plugins import non-test modules (for example doctesting).

     If this step succeeds, the module is returned.

     For test modules, unless they are reachable from :py:data:`sys.path`, this step will fail.

  2. If the previous step fails, we import the module directly using ``importlib`` facilities, which lets us import it without
     changing :py:data:`sys.path`.

     Because Python requires the module to also be available in :py:data:`sys.modules`, pytest derives a unique name for it based
     on its relative location from the ``rootdir``, and adds the module to :py:data:`sys.modules`.

     For example, ``tests/core/test_models.py`` will end up being imported as the module ``tests.core.test_models``.

  .. versionadded:: 6.0

.. note::

    Initially we intended to make ``importlib`` the default in future releases, however it is clear now that
    it has its own set of drawbacks so the default will remain ``prepend`` for the foreseeable future.

.. note::

    By default, pytest will not attempt to resolve namespace packages automatically, but that can
    be changed via the :confval:`consider_namespace_packages` configuration variable.

.. seealso::

    The :confval:`pythonpath` configuration variable.

    The :confval:`consider_namespace_packages` configuration variable.

    :ref:`test layout`.


``prepend`` and ``append`` import modes scenarios
-------------------------------------------------

Here's a list of scenarios when using ``prepend`` or ``append`` import modes where pytest needs to
change :py:data:`sys.path` in order to import test modules or ``conftest.py`` files, and the issues users
might encounter because of that.

Test modules / ``conftest.py`` files inside packages
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Consider this file and directory layout::

    root/
    |- foo/
       |- __init__.py
       |- conftest.py
       |- bar/
          |- __init__.py
          |- tests/
             |- __init__.py
             |- test_foo.py


When executing:

.. code-block:: bash

    pytest root/

pytest will find ``foo/bar/tests/test_foo.py`` and realize it is part of a package given that
there's an ``__init__.py`` file in the same folder. It will then search upwards until it can find the
last folder which still contains an ``__init__.py`` file in order to find the package *root* (in
this case ``foo/``). To load the module, it will insert ``root/``  to the front of
:py:data:`sys.path` (if not there already) in order to load
``test_foo.py`` as the *module* ``foo.bar.tests.test_foo``.

The same logic applies to the ``conftest.py`` file: it will be imported as ``foo.conftest`` module.

Preserving the full package name is important when tests live in a package to avoid problems
and allow test modules to have duplicated names. This is also discussed in details in
:ref:`test discovery`.

Standalone test modules / ``conftest.py`` files
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Consider this file and directory layout::

    root/
    |- foo/
       |- conftest.py
       |- bar/
          |- tests/
             |- test_foo.py


When executing:

.. code-block:: bash

    pytest root/

pytest will find ``foo/bar/tests/test_foo.py`` and realize it is NOT part of a package given that
there's no ``__init__.py`` file in the same folder. It will then add ``root/foo/bar/tests`` to
:py:data:`sys.path` in order to import ``test_foo.py`` as the *module* ``test_foo``. The same is done
with the ``conftest.py`` file by adding ``root/foo`` to :py:data:`sys.path` to import it as ``conftest``.

For this reason this layout cannot have test modules with the same name, as they all will be
imported in the global import namespace.

This is also discussed in details in :ref:`test discovery`.

.. _`pytest vs python -m pytest`:

Invoking ``pytest`` versus ``python -m pytest``
-----------------------------------------------

Running pytest with ``pytest [...]`` instead of ``python -m pytest [...]`` yields nearly
equivalent behaviour, except that the latter will add the current directory to :py:data:`sys.path`, which
is standard ``python`` behavior.

See also :ref:`invoke-python`.
