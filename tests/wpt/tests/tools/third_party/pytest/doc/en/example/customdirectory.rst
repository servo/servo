.. _`custom directory collectors`:

Using a custom directory collector
====================================================

By default, pytest collects directories using :class:`pytest.Package`, for directories with ``__init__.py`` files,
and :class:`pytest.Dir` for other directories.
If you want to customize how a directory is collected, you can write your own :class:`pytest.Directory` collector,
and use :hook:`pytest_collect_directory` to hook it up.

.. _`directory manifest plugin`:

A basic example for a directory manifest file
--------------------------------------------------------------

Suppose you want to customize how collection is done on a per-directory basis.
Here is an example ``conftest.py`` plugin that allows directories to contain a ``manifest.json`` file,
which defines how the collection should be done for the directory.
In this example, only a simple list of files is supported,
however you can imagine adding other keys, such as exclusions and globs.

.. include:: customdirectory/conftest.py
    :literal:

You can create a ``manifest.json`` file and some test files:

.. include:: customdirectory/tests/manifest.json
    :literal:

.. include:: customdirectory/tests/test_first.py
    :literal:

.. include:: customdirectory/tests/test_second.py
    :literal:

.. include:: customdirectory/tests/test_third.py
    :literal:

An you can now execute the test specification:

.. code-block:: pytest

    customdirectory $ pytest
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project/customdirectory
    configfile: pytest.ini
    collected 2 items

    tests/test_first.py .                                                [ 50%]
    tests/test_second.py .                                               [100%]

    ============================ 2 passed in 0.12s =============================

.. regendoc:wipe

Notice how ``test_three.py`` was not executed, because it is not listed in the manifest.

You can verify that your custom collector appears in the collection tree:

.. code-block:: pytest

    customdirectory $ pytest --collect-only
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project/customdirectory
    configfile: pytest.ini
    collected 2 items

    <Dir customdirectory>
      <ManifestDirectory tests>
        <Module test_first.py>
          <Function test_1>
        <Module test_second.py>
          <Function test_2>

    ======================== 2 tests collected in 0.12s ========================
