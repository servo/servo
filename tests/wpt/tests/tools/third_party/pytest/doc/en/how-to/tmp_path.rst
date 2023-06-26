
.. _`tmp_path handling`:
.. _tmp_path:

How to use temporary directories and files in tests
===================================================

The ``tmp_path`` fixture
------------------------

You can use the ``tmp_path`` fixture which will
provide a temporary directory unique to the test invocation,
created in the `base temporary directory`_.

``tmp_path`` is a :class:`pathlib.Path` object. Here is an example test usage:

.. code-block:: python

    # content of test_tmp_path.py
    CONTENT = "content"


    def test_create_file(tmp_path):
        d = tmp_path / "sub"
        d.mkdir()
        p = d / "hello.txt"
        p.write_text(CONTENT)
        assert p.read_text() == CONTENT
        assert len(list(tmp_path.iterdir())) == 1
        assert 0

Running this would result in a passed test except for the last
``assert 0`` line which we use to look at values:

.. code-block:: pytest

    $ pytest test_tmp_path.py
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-7.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    collected 1 item

    test_tmp_path.py F                                                   [100%]

    ================================= FAILURES =================================
    _____________________________ test_create_file _____________________________

    tmp_path = PosixPath('PYTEST_TMPDIR/test_create_file0')

        def test_create_file(tmp_path):
            d = tmp_path / "sub"
            d.mkdir()
            p = d / "hello.txt"
            p.write_text(CONTENT)
            assert p.read_text() == CONTENT
            assert len(list(tmp_path.iterdir())) == 1
    >       assert 0
    E       assert 0

    test_tmp_path.py:11: AssertionError
    ========================= short test summary info ==========================
    FAILED test_tmp_path.py::test_create_file - assert 0
    ============================ 1 failed in 0.12s =============================

.. _`tmp_path_factory example`:

The ``tmp_path_factory`` fixture
--------------------------------

The ``tmp_path_factory`` is a session-scoped fixture which can be used
to create arbitrary temporary directories from any other fixture or test.

For example, suppose your test suite needs a large image on disk, which is
generated procedurally. Instead of computing the same image for each test
that uses it into its own ``tmp_path``, you can generate it once per-session
to save time:

.. code-block:: python

    # contents of conftest.py
    import pytest


    @pytest.fixture(scope="session")
    def image_file(tmp_path_factory):
        img = compute_expensive_image()
        fn = tmp_path_factory.mktemp("data") / "img.png"
        img.save(fn)
        return fn


    # contents of test_image.py
    def test_histogram(image_file):
        img = load_image(image_file)
        # compute and test histogram

See :ref:`tmp_path_factory API <tmp_path_factory factory api>` for details.

.. _`tmpdir and tmpdir_factory`:
.. _tmpdir:

The ``tmpdir`` and ``tmpdir_factory`` fixtures
---------------------------------------------------

The ``tmpdir`` and ``tmpdir_factory`` fixtures are similar to ``tmp_path``
and ``tmp_path_factory``, but use/return legacy `py.path.local`_ objects
rather than standard :class:`pathlib.Path` objects. These days, prefer to
use ``tmp_path`` and ``tmp_path_factory``.

See :fixture:`tmpdir <tmpdir>` :fixture:`tmpdir_factory <tmpdir_factory>`
API for details.


.. _`base temporary directory`:

The default base temporary directory
-----------------------------------------------

Temporary directories are by default created as sub-directories of
the system temporary directory.  The base name will be ``pytest-NUM`` where
``NUM`` will be incremented with each test run.  Moreover, entries older
than 3 temporary directories will be removed.

You can override the default temporary directory setting like this:

.. code-block:: bash

    pytest --basetemp=mydir

.. warning::

    The contents of ``mydir`` will be completely removed, so make sure to use a directory
    for that purpose only.

When distributing tests on the local machine using ``pytest-xdist``, care is taken to
automatically configure a basetemp directory for the sub processes such that all temporary
data lands below a single per-test run basetemp directory.

.. _`py.path.local`: https://py.readthedocs.io/en/latest/path.html
