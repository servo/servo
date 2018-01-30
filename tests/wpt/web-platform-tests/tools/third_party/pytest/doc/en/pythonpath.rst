.. _pythonpath:

pytest import mechanisms and ``sys.path``/``PYTHONPATH``
========================================================

Here's a list of scenarios where pytest may need to change ``sys.path`` in order
to import test modules or ``conftest.py`` files.

Test modules / ``conftest.py`` files inside packages
----------------------------------------------------

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


When executing::

    pytest root/



pytest will find ``foo/bar/tests/test_foo.py`` and realize it is part of a package given that
there's an ``__init__.py`` file in the same folder. It will then search upwards until it can find the
last folder which still contains an ``__init__.py`` file in order to find the package *root* (in
this case ``foo/``). To load the module, it will insert ``root/``  to the front of
``sys.path`` (if not there already) in order to load
``test_foo.py`` as the *module* ``foo.bar.tests.test_foo``.

The same logic applies to the ``conftest.py`` file: it will be imported as ``foo.conftest`` module.

Preserving the full package name is important when tests live in a package to avoid problems
and allow test modules to have duplicated names. This is also discussed in details in
:ref:`test discovery`.

Standalone test modules / ``conftest.py`` files
-----------------------------------------------

Consider this file and directory layout::

    root/
    |- foo/
       |- conftest.py
       |- bar/
          |- tests/
             |- test_foo.py


When executing::

    pytest root/

pytest will find ``foo/bar/tests/test_foo.py`` and realize it is NOT part of a package given that
there's no ``__init__.py`` file in the same folder. It will then add ``root/foo/bar/tests`` to
``sys.path`` in order to import ``test_foo.py`` as the *module* ``test_foo``. The same is done
with the ``conftest.py`` file by adding ``root/foo`` to ``sys.path`` to import it as ``conftest``.

For this reason this layout cannot have test modules with the same name, as they all will be
imported in the global import namespace.

This is also discussed in details in :ref:`test discovery`.

Invoking ``pytest`` versus ``python -m pytest``
-----------------------------------------------

Running pytest with ``python -m pytest [...]`` instead of ``pytest [...]`` yields nearly
equivalent behaviour, except that the former call will add the current directory to ``sys.path``.
See also :ref:`cmdline`.
