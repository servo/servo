
.. _`non-python tests`:

Working with non-python tests
====================================================

.. _`yaml plugin`:

A basic example for specifying tests in Yaml files
--------------------------------------------------------------

.. _`pytest-yamlwsgi`: http://bitbucket.org/aafshar/pytest-yamlwsgi/src/tip/pytest_yamlwsgi.py
.. _`PyYAML`: https://pypi.org/project/PyYAML/

Here is an example ``conftest.py`` (extracted from Ali Afshar's special purpose `pytest-yamlwsgi`_ plugin).   This ``conftest.py`` will  collect ``test*.yaml`` files and will execute the yaml-formatted content as custom tests:

.. include:: nonpython/conftest.py
    :literal:

You can create a simple example file:

.. include:: nonpython/test_simple.yaml
    :literal:

and if you installed `PyYAML`_ or a compatible YAML-parser you can
now execute the test specification:

.. code-block:: pytest

    nonpython $ pytest test_simple.yaml
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-6.x.y, py-1.x.y, pluggy-0.x.y
    cachedir: $PYTHON_PREFIX/.pytest_cache
    rootdir: $REGENDOC_TMPDIR/nonpython
    collected 2 items

    test_simple.yaml F.                                                  [100%]

    ================================= FAILURES =================================
    ______________________________ usecase: hello ______________________________
    usecase execution failed
       spec failed: 'some': 'other'
       no further details known at this point.
    ========================= short test summary info ==========================
    FAILED test_simple.yaml::hello
    ======================= 1 failed, 1 passed in 0.12s ========================

.. regendoc:wipe

You get one dot for the passing ``sub1: sub1`` check and one failure.
Obviously in the above ``conftest.py`` you'll want to implement a more
interesting interpretation of the yaml-values.  You can easily write
your own domain specific testing language this way.

.. note::

    ``repr_failure(excinfo)`` is called for representing test failures.
    If you create custom collection nodes you can return an error
    representation string of your choice.  It
    will be reported as a (red) string.

``reportinfo()`` is used for representing the test location and is also
consulted when reporting in ``verbose`` mode:

.. code-block:: pytest

    nonpython $ pytest -v
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-6.x.y, py-1.x.y, pluggy-0.x.y -- $PYTHON_PREFIX/bin/python
    cachedir: $PYTHON_PREFIX/.pytest_cache
    rootdir: $REGENDOC_TMPDIR/nonpython
    collecting ... collected 2 items

    test_simple.yaml::hello FAILED                                       [ 50%]
    test_simple.yaml::ok PASSED                                          [100%]

    ================================= FAILURES =================================
    ______________________________ usecase: hello ______________________________
    usecase execution failed
       spec failed: 'some': 'other'
       no further details known at this point.
    ========================= short test summary info ==========================
    FAILED test_simple.yaml::hello
    ======================= 1 failed, 1 passed in 0.12s ========================

.. regendoc:wipe

While developing your custom test collection and execution it's also
interesting to just look at the collection tree:

.. code-block:: pytest

    nonpython $ pytest --collect-only
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-6.x.y, py-1.x.y, pluggy-0.x.y
    cachedir: $PYTHON_PREFIX/.pytest_cache
    rootdir: $REGENDOC_TMPDIR/nonpython
    collected 2 items

    <Package nonpython>
      <YamlFile test_simple.yaml>
        <YamlItem hello>
        <YamlItem ok>

    ========================== no tests ran in 0.12s ===========================
