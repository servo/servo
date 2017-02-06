.. image:: http://pytest.org/latest/_static/pytest1.png
   :target: http://pytest.org
   :align: center
   :alt: pytest

------

.. image:: https://img.shields.io/pypi/v/pytest.svg
   :target: https://pypi.python.org/pypi/pytest
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
    def func(x):
        return x + 1

    def test_answer():
        assert func(3) == 5


To execute it::

    $ py.test
    ======= test session starts ========
    platform linux -- Python 3.4.3, pytest-2.8.5, py-1.4.31, pluggy-0.3.1    
    collected 1 items

    test_sample.py F

    ======= FAILURES ========
    _______ test_answer ________

        def test_answer():
    >       assert func(3) == 5
    E       assert 4 == 5
    E        +  where 4 = func(3)

    test_sample.py:5: AssertionError
    ======= 1 failed in 0.12 seconds ========

Due to ``py.test``'s detailed assertion introspection, only plain ``assert`` statements are used. See `getting-started <http://pytest.org/latest/getting-started.html#our-first-test-run>`_ for more examples.
        

Features
--------

- Detailed info on failing `assert statements <http://pytest.org/latest/assert.html>`_ (no need to remember ``self.assert*`` names);

- `Auto-discovery
  <http://pytest.org/latest/goodpractices.html#python-test-discovery>`_
  of test modules and functions;

- `Modular fixtures <http://pytest.org/latest/fixture.html>`_  for
  managing small or parametrized long-lived test resources;

- Can run `unittest <http://pytest.org/latest/unittest.html>`_ (or trial),
  `nose <http://pytest.org/latest/nose.html>`_ test suites out of the box;

- Python2.6+, Python3.2+, PyPy-2.3, Jython-2.5 (untested);

- Rich plugin architecture, with over 150+ `external plugins <http://pytest.org/latest/plugins.html#installing-external-plugins-searching>`_ and thriving community;


Documentation
-------------

For full documentation, including installation, tutorials and PDF documents, please see http://pytest.org.


Bugs/Requests
-------------

Please use the `GitHub issue tracker <https://github.com/pytest-dev/pytest/issues>`_ to submit bugs or request features.


Changelog
---------

Consult the `Changelog <http://pytest.org/latest/changelog.html>`_ page for fixes and enhancements of each version.


License
-------

Copyright Holger Krekel and others, 2004-2016.

Distributed under the terms of the `MIT`_ license, pytest is free and open source software.

.. _`MIT`: https://github.com/pytest-dev/pytest/blob/master/LICENSE
