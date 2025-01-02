.. _`external plugins`:
.. _`extplugins`:
.. _`using plugins`:

How to install and use plugins
===============================

This section talks about installing and using third party plugins.
For writing your own plugins, please refer to :ref:`writing-plugins`.

Installing a third party plugin can be easily done with ``pip``:

.. code-block:: bash

    pip install pytest-NAME
    pip uninstall pytest-NAME

If a plugin is installed, ``pytest`` automatically finds and integrates it,
there is no need to activate it.

Here is a little annotated list for some popular plugins:

* :pypi:`pytest-django`: write tests
  for `django <https://docs.djangoproject.com/>`_ apps, using pytest integration.

* :pypi:`pytest-twisted`: write tests
  for `twisted <https://twistedmatrix.com/>`_ apps, starting a reactor and
  processing deferreds from test functions.

* :pypi:`pytest-cov`:
  coverage reporting, compatible with distributed testing

* :pypi:`pytest-xdist`:
  to distribute tests to CPUs and remote hosts, to run in boxed
  mode which allows to survive segmentation faults, to run in
  looponfailing mode, automatically re-running failing tests
  on file changes.

* :pypi:`pytest-instafail`:
  to report failures while the test run is happening.

* :pypi:`pytest-bdd`:
  to write tests using behaviour-driven testing.

* :pypi:`pytest-timeout`:
  to timeout tests based on function marks or global definitions.

* :pypi:`pytest-pep8`:
  a ``--pep8`` option to enable PEP8 compliance checking.

* :pypi:`pytest-flakes`:
  check source code with pyflakes.

* :pypi:`allure-pytest`:
  report test results via `allure-framework <https://github.com/allure-framework/>`_.

To see a complete list of all plugins with their latest testing
status against different pytest and Python versions, please visit
:ref:`plugin-list`.

You may also discover more plugins through a `pytest- pypi.org search`_.

.. _`pytest- pypi.org search`: https://pypi.org/search/?q=pytest-


.. _`available installable plugins`:

Requiring/Loading plugins in a test module or conftest file
-----------------------------------------------------------

You can require plugins in a test module or a conftest file using :globalvar:`pytest_plugins`:

.. code-block:: python

    pytest_plugins = ("myapp.testsupport.myplugin",)

When the test module or conftest plugin is loaded the specified plugins
will be loaded as well.

.. note::

    Requiring plugins using a ``pytest_plugins`` variable in non-root
    ``conftest.py`` files is deprecated. See
    :ref:`full explanation <requiring plugins in non-root conftests>`
    in the Writing plugins section.

.. note::
   The name ``pytest_plugins`` is reserved and should not be used as a
   name for a custom plugin module.


.. _`findpluginname`:

Finding out which plugins are active
------------------------------------

If you want to find out which plugins are active in your
environment you can type:

.. code-block:: bash

    pytest --trace-config

and will get an extended test header which shows activated plugins
and their names. It will also print local plugins aka
:ref:`conftest.py <conftest.py plugins>` files when they are loaded.

.. _`cmdunregister`:

Deactivating / unregistering a plugin by name
---------------------------------------------

You can prevent plugins from loading or unregister them:

.. code-block:: bash

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
