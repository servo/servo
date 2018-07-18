
.. _`tmpdir handling`:
.. _tmpdir:

Temporary directories and files
================================================

The 'tmpdir' fixture
--------------------

You can use the ``tmpdir`` fixture which will
provide a temporary directory unique to the test invocation,
created in the `base temporary directory`_.

``tmpdir`` is a `py.path.local`_ object which offers ``os.path`` methods
and more.  Here is an example test usage::

    # content of test_tmpdir.py
    import os
    def test_create_file(tmpdir):
        p = tmpdir.mkdir("sub").join("hello.txt")
        p.write("content")
        assert p.read() == "content"
        assert len(tmpdir.listdir()) == 1
        assert 0

Running this would result in a passed test except for the last
``assert 0`` line which we use to look at values::

    $ pytest test_tmpdir.py
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-3.x.y, py-1.x.y, pluggy-0.x.y
    rootdir: $REGENDOC_TMPDIR, inifile:
    collected 1 item

    test_tmpdir.py F                                                     [100%]

    ================================= FAILURES =================================
    _____________________________ test_create_file _____________________________

    tmpdir = local('PYTEST_TMPDIR/test_create_file0')

        def test_create_file(tmpdir):
            p = tmpdir.mkdir("sub").join("hello.txt")
            p.write("content")
            assert p.read() == "content"
            assert len(tmpdir.listdir()) == 1
    >       assert 0
    E       assert 0

    test_tmpdir.py:7: AssertionError
    ========================= 1 failed in 0.12 seconds =========================

.. _`tmpdir factory example`:

The 'tmpdir_factory' fixture
----------------------------

.. versionadded:: 2.8

The ``tmpdir_factory`` is a session-scoped fixture which can be used
to create arbitrary temporary directories from any other fixture or test.

For example, suppose your test suite needs a large image on disk, which is
generated procedurally. Instead of computing the same image for each test
that uses it into its own ``tmpdir``, you can generate it once per-session
to save time:

.. code-block:: python

    # contents of conftest.py
    import pytest


    @pytest.fixture(scope="session")
    def image_file(tmpdir_factory):
        img = compute_expensive_image()
        fn = tmpdir_factory.mktemp("data").join("img.png")
        img.save(str(fn))
        return fn


    # contents of test_image.py
    def test_histogram(image_file):
        img = load_image(image_file)
        # compute and test histogram

See :ref:`tmpdir_factory API <tmpdir factory api>` for details.


.. _`base temporary directory`:

The default base temporary directory
-----------------------------------------------

Temporary directories are by default created as sub-directories of
the system temporary directory.  The base name will be ``pytest-NUM`` where
``NUM`` will be incremented with each test run.  Moreover, entries older
than 3 temporary directories will be removed.

You can override the default temporary directory setting like this::

    pytest --basetemp=mydir

When distributing tests on the local machine, ``pytest`` takes care to
configure a basetemp directory for the sub processes such that all temporary
data lands below a single per-test run basetemp directory.

.. _`py.path.local`: https://py.readthedocs.io/en/latest/path.html
