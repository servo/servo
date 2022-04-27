.. _`noseintegration`:

How to run tests written for nose
=======================================

``pytest`` has basic support for running tests written for nose_.

.. _nosestyle:

Usage
-------------

After :ref:`installation` type:

.. code-block:: bash

    python setup.py develop  # make sure tests can import our package
    pytest  # instead of 'nosetests'

and you should be able to run your nose style tests and
make use of pytest's capabilities.

Supported nose Idioms
----------------------

* setup and teardown at module/class/method level
* SkipTest exceptions and markers
* setup/teardown decorators
* ``__test__`` attribute on modules/classes/functions
* general usage of nose utilities

Unsupported idioms / known issues
----------------------------------

- unittest-style ``setUp, tearDown, setUpClass, tearDownClass``
  are recognized only on ``unittest.TestCase`` classes but not
  on plain classes.  ``nose`` supports these methods also on plain
  classes but pytest deliberately does not.  As nose and pytest already
  both support ``setup_class, teardown_class, setup_method, teardown_method``
  it doesn't seem useful to duplicate the unittest-API like nose does.
  If you however rather think pytest should support the unittest-spelling on
  plain classes please post to :issue:`377`.

- nose imports test modules with the same import path (e.g.
  ``tests.test_mode``) but different file system paths
  (e.g. ``tests/test_mode.py`` and ``other/tests/test_mode.py``)
  by extending sys.path/import semantics.   pytest does not do that
  but there is discussion in :issue:`268` for adding some support.  Note that
  `nose2 choose to avoid this sys.path/import hackery <https://nose2.readthedocs.io/en/latest/differences.html#test-discovery-and-loading>`_.

  If you place a conftest.py file in the root directory of your project
  (as determined by pytest) pytest will run tests "nose style" against
  the code below that directory by adding it to your ``sys.path`` instead of
  running against your installed code.

  You may find yourself wanting to do this if you ran ``python setup.py install``
  to set up your project, as opposed to ``python setup.py develop`` or any of
  the package manager equivalents.  Installing with develop in a
  virtual environment like tox is recommended over this pattern.

- nose-style doctests are not collected and executed correctly,
  also doctest fixtures don't work.

- no nose-configuration is recognized.

- ``yield``-based methods are unsupported as of pytest 4.1.0.  They are
  fundamentally incompatible with pytest because they don't support fixtures
  properly since collection and test execution are separated.

Migrating from nose to pytest
------------------------------

`nose2pytest <https://github.com/pytest-dev/nose2pytest>`_ is a Python script
and pytest plugin to help convert Nose-based tests into pytest-based tests.
Specifically, the script transforms nose.tools.assert_* function calls into
raw assert statements, while preserving format of original arguments
as much as possible.

.. _nose: https://nose.readthedocs.io/en/latest/
