.. image:: http://docs.pytest.org/en/latest/_static/pytest1.png
   :target: http://docs.pytest.org
   :align: center
   :alt: pytest

------

.. image:: https://img.shields.io/pypi/v/pytest.svg
    :target: https://pypi.python.org/pypi/pytest

.. image:: https://anaconda.org/conda-forge/pytest/badges/version.svg
    :target: https://anaconda.org/conda-forge/pytest

.. image:: https://img.shields.io/pypi/pyversions/pytest.svg
    :target: https://pypi.python.org/pypi/pytest

.. image:: https://img.shields.io/coveralls/pytest-dev/pytest/master.svg
    :target: https://coveralls.io/r/pytest-dev/pytest

.. image:: https://travis-ci.org/pytest-dev/pytest.svg?branch=master
    :target: https://travis-ci.org/pytest-dev/pytest

.. image:: https://ci.appveyor.com/api/projects/status/mrgbjaua7t33pg6b?svg=true
    :target: https://ci.appveyor.com/project/pytestbot/pytest

The ``pytest`` framework makes it easy to write small tests, yet
scales to support complex functional testing for applications and libraries.

An example of a simple test:

.. code-block:: python

    # content of test_sample.py
    def inc(x):
        return x + 1

    def test_answer():
        assert inc(3) == 5


To execute it::

    $ pytest
    ============================= test session starts =============================
    collected 1 items

    test_sample.py F

    ================================== FAILURES ===================================
    _________________________________ test_answer _________________________________

        def test_answer():
    >       assert inc(3) == 5
    E       assert 4 == 5
    E        +  where 4 = inc(3)

    test_sample.py:5: AssertionError
    ========================== 1 failed in 0.04 seconds ===========================


Due to ``pytest``'s detailed assertion introspection, only plain ``assert`` statements are used. See `getting-started <http://docs.pytest.org/en/latest/getting-started.html#our-first-test-run>`_ for more examples.


Features
--------

- Detailed info on failing `assert statements <http://docs.pytest.org/en/latest/assert.html>`_ (no need to remember ``self.assert*`` names);

- `Auto-discovery
  <http://docs.pytest.org/en/latest/goodpractices.html#python-test-discovery>`_
  of test modules and functions;

- `Modular fixtures <http://docs.pytest.org/en/latest/fixture.html>`_ for
  managing small or parametrized long-lived test resources;

- Can run `unittest <http://docs.pytest.org/en/latest/unittest.html>`_ (or trial),
  `nose <http://docs.pytest.org/en/latest/nose.html>`_ test suites out of the box;

- Python 2.7, Python 3.4+, PyPy 2.3, Jython 2.5 (untested);

- Rich plugin architecture, with over 315+ `external plugins <http://plugincompat.herokuapp.com>`_ and thriving community;


Documentation
-------------

For full documentation, including installation, tutorials and PDF documents, please see http://docs.pytest.org.


Bugs/Requests
-------------

Please use the `GitHub issue tracker <https://github.com/pytest-dev/pytest/issues>`_ to submit bugs or request features.


Changelog
---------

Consult the `Changelog <http://docs.pytest.org/en/latest/changelog.html>`__ page for fixes and enhancements of each version.


License
-------

Copyright Holger Krekel and others, 2004-2017.

Distributed under the terms of the `MIT`_ license, pytest is free and open source software.

.. _`MIT`: https://github.com/pytest-dev/pytest/blob/master/LICENSE
