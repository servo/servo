
.. _`non-python tests`:

Working with non-python tests
====================================================

.. _`yaml plugin`:

A basic example for specifying tests in Yaml files
--------------------------------------------------------------

.. _`pytest-yamlwsgi`: http://bitbucket.org/aafshar/pytest-yamlwsgi/src/tip/pytest_yamlwsgi.py
.. _`PyYAML`: http://pypi.python.org/pypi/PyYAML/

Here is an example ``conftest.py`` (extracted from Ali Afshnars special purpose `pytest-yamlwsgi`_ plugin).   This ``conftest.py`` will  collect ``test*.yml`` files and will execute the yaml-formatted content as custom tests:

.. include:: nonpython/conftest.py
    :literal:

You can create a simple example file:

.. include:: nonpython/test_simple.yml
    :literal:

and if you installed `PyYAML`_ or a compatible YAML-parser you can
now execute the test specification::

    nonpython $ py.test test_simple.yml
    ======= test session starts ========
    platform linux -- Python 3.4.0, pytest-2.9.1, py-1.4.31, pluggy-0.3.1
    rootdir: $REGENDOC_TMPDIR/nonpython, inifile: 
    collected 2 items
    
    test_simple.yml F.
    
    ======= FAILURES ========
    _______ usecase: hello ________
    usecase execution failed
       spec failed: 'some': 'other'
       no further details known at this point.
    ======= 1 failed, 1 passed in 0.12 seconds ========

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
consulted when reporting in ``verbose`` mode::

    nonpython $ py.test -v
    ======= test session starts ========
    platform linux -- Python 3.4.0, pytest-2.9.1, py-1.4.31, pluggy-0.3.1 -- $PYTHON_PREFIX/bin/python3.4
    cachedir: .cache
    rootdir: $REGENDOC_TMPDIR/nonpython, inifile: 
    collecting ... collected 2 items
    
    test_simple.yml::hello FAILED
    test_simple.yml::ok PASSED
    
    ======= FAILURES ========
    _______ usecase: hello ________
    usecase execution failed
       spec failed: 'some': 'other'
       no further details known at this point.
    ======= 1 failed, 1 passed in 0.12 seconds ========

.. regendoc:wipe

While developing your custom test collection and execution it's also
interesting to just look at the collection tree::

    nonpython $ py.test --collect-only
    ======= test session starts ========
    platform linux -- Python 3.4.0, pytest-2.9.1, py-1.4.31, pluggy-0.3.1
    rootdir: $REGENDOC_TMPDIR/nonpython, inifile: 
    collected 2 items
    <YamlFile 'test_simple.yml'>
      <YamlItem 'hello'>
      <YamlItem 'ok'>
    
    ======= no tests ran in 0.12 seconds ========
