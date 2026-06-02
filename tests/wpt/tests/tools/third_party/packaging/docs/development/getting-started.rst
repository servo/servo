Getting started
===============

Working on packaging requires the installation of a small number of
development dependencies. To see what dependencies are required to
run the tests manually, please look at the ``noxfile.py`` file.

Running tests
~~~~~~~~~~~~~

The packaging unit tests are found in the ``tests/`` directory and are
designed to be run using `pytest`_. `pytest`_ will discover the tests
automatically, so all you have to do is:

.. code-block:: console

    $ python -m pytest
    ...
    29204 passed, 4 skipped, 1 xfailed in 83.98 seconds

This runs the tests with the default Python interpreter. This also allows
you to run select tests instead of the entire test suite.

You can also verify that the tests pass on other supported Python interpreters.
For this we use `nox`_, which will automatically create a `virtualenv`_ for
each supported Python version and run the tests. For example:

.. code-block:: console

    $ nox -s tests
    ...
    nox > Ran multiple sessions:
    nox > * tests-3.6: success
    nox > * tests-3.7: success
    nox > * tests-3.8: success
    nox > * tests-3.9: success
    nox > * tests-pypy3: skipped

You may not have all the required Python versions installed, in which case you
will see one or more ``InterpreterNotFound`` errors.

Running linters
~~~~~~~~~~~~~~~

If you wish to run the linting rules, you may use `pre-commit`_ or run
``nox -s lint``.

.. code-block:: console

    $ nox -s lint
    ...
    nox > Session lint was successful.

Building documentation
~~~~~~~~~~~~~~~~~~~~~~

packaging documentation is stored in the ``docs/`` directory. It is
written in `reStructured Text`_ and rendered using `Sphinx`_.

Use `nox`_ to build the documentation. For example:

.. code-block:: console

    $ nox -s docs
    ...
    nox > Session docs was successful.

The HTML documentation index can now be found at
``docs/_build/html/index.html``.

.. _`pytest`: https://pypi.org/project/pytest/
.. _`nox`: https://pypi.org/project/nox/
.. _`virtualenv`: https://pypi.org/project/virtualenv/
.. _`pip`: https://pypi.org/project/pip/
.. _`sphinx`: https://pypi.org/project/Sphinx/
.. _`reStructured Text`: http://sphinx-doc.org/rest.html
.. _`pre-commit`: https://pre-commit.com
