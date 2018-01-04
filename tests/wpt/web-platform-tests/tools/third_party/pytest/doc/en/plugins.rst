.. _`external plugins`:
.. _`extplugins`:
.. _`using plugins`:

Installing and Using plugins
============================

This section talks about installing and using third party plugins.
For writing your own plugins, please refer to :ref:`writing-plugins`.

Installing a third party plugin can be easily done with ``pip``::

    pip install pytest-NAME
    pip uninstall pytest-NAME

If a plugin is installed, ``pytest`` automatically finds and integrates it,
there is no need to activate it.

Here is a little annotated list for some popular plugins:

.. _`django`: https://www.djangoproject.com/

* `pytest-django <http://pypi.python.org/pypi/pytest-django>`_: write tests
  for `django`_ apps, using pytest integration.

* `pytest-twisted <http://pypi.python.org/pypi/pytest-twisted>`_: write tests
  for `twisted <http://twistedmatrix.com>`_ apps, starting a reactor and
  processing deferreds from test functions.

* `pytest-cov <http://pypi.python.org/pypi/pytest-cov>`_:
  coverage reporting, compatible with distributed testing

* `pytest-xdist <http://pypi.python.org/pypi/pytest-xdist>`_:
  to distribute tests to CPUs and remote hosts, to run in boxed
  mode which allows to survive segmentation faults, to run in
  looponfailing mode, automatically re-running failing tests
  on file changes.

* `pytest-instafail <http://pypi.python.org/pypi/pytest-instafail>`_:
  to report failures while the test run is happening.

* `pytest-bdd <http://pypi.python.org/pypi/pytest-bdd>`_ and
  `pytest-konira <http://pypi.python.org/pypi/pytest-konira>`_
  to write tests using behaviour-driven testing.

* `pytest-timeout <http://pypi.python.org/pypi/pytest-timeout>`_:
  to timeout tests based on function marks or global definitions.

* `pytest-pep8 <http://pypi.python.org/pypi/pytest-pep8>`_:
  a ``--pep8`` option to enable PEP8 compliance checking.

* `pytest-flakes <https://pypi.python.org/pypi/pytest-flakes>`_:
  check source code with pyflakes.

* `oejskit <http://pypi.python.org/pypi/oejskit>`_:
  a plugin to run javascript unittests in live browsers.

To see a complete list of all plugins with their latest testing
status against different pytest and Python versions, please visit
`plugincompat <http://plugincompat.herokuapp.com/>`_.

You may also discover more plugins through a `pytest- pypi.python.org search`_.

.. _`available installable plugins`:
.. _`pytest- pypi.python.org search`: http://pypi.python.org/pypi?%3Aaction=search&term=pytest-&submit=search


Requiring/Loading plugins in a test module or conftest file
-----------------------------------------------------------

You can require plugins in a test module or a conftest file like this::

    pytest_plugins = "myapp.testsupport.myplugin",

When the test module or conftest plugin is loaded the specified plugins
will be loaded as well.

    pytest_plugins = "myapp.testsupport.myplugin"

which will import the specified module as a ``pytest`` plugin.

.. _`findpluginname`:

Finding out which plugins are active
------------------------------------

If you want to find out which plugins are active in your
environment you can type::

    pytest --trace-config

and will get an extended test header which shows activated plugins
and their names. It will also print local plugins aka
:ref:`conftest.py <conftest.py plugins>` files when they are loaded.

.. _`cmdunregister`:

Deactivating / unregistering a plugin by name
---------------------------------------------

You can prevent plugins from loading or unregister them::

    pytest -p no:NAME

This means that any subsequent try to activate/load the named
plugin will not work.

If you want to unconditionally disable a plugin for a project, you can add
this option to your ``pytest.ini`` file:

.. code-block:: ini

      [pytest]
      addopts = -p no:NAME

Alternatively to disable it only in certain environments (for example in a
CI server), you can set ``PYTEST_ADDOPTS`` environment variable to
``-p no:name``.

See :ref:`findpluginname` for how to obtain the name of a plugin.

.. _`builtin plugins`:

Pytest default plugin reference
-------------------------------


You can find the source code for the following plugins
in the `pytest repository <https://github.com/pytest-dev/pytest>`_.

.. autosummary::

    _pytest.assertion
    _pytest.cacheprovider
    _pytest.capture
    _pytest.config
    _pytest.doctest
    _pytest.helpconfig
    _pytest.junitxml
    _pytest.mark
    _pytest.monkeypatch
    _pytest.nose
    _pytest.pastebin
    _pytest.debugging
    _pytest.pytester
    _pytest.python
    _pytest.recwarn
    _pytest.resultlog
    _pytest.runner
    _pytest.main
    _pytest.skipping
    _pytest.terminal
    _pytest.tmpdir
    _pytest.unittest
