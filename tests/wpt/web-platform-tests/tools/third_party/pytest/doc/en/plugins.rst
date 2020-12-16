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

* `pytest-django <https://pypi.org/project/pytest-django/>`_: write tests
  for `django`_ apps, using pytest integration.

* `pytest-twisted <https://pypi.org/project/pytest-twisted/>`_: write tests
  for `twisted <http://twistedmatrix.com>`_ apps, starting a reactor and
  processing deferreds from test functions.

* `pytest-cov <https://pypi.org/project/pytest-cov/>`__:
  coverage reporting, compatible with distributed testing

* `pytest-xdist <https://pypi.org/project/pytest-xdist/>`_:
  to distribute tests to CPUs and remote hosts, to run in boxed
  mode which allows to survive segmentation faults, to run in
  looponfailing mode, automatically re-running failing tests
  on file changes.

* `pytest-instafail <https://pypi.org/project/pytest-instafail/>`_:
  to report failures while the test run is happening.

* `pytest-bdd <https://pypi.org/project/pytest-bdd/>`_ and
  `pytest-konira <https://pypi.org/project/pytest-konira/>`_
  to write tests using behaviour-driven testing.

* `pytest-timeout <https://pypi.org/project/pytest-timeout/>`_:
  to timeout tests based on function marks or global definitions.

* `pytest-pep8 <https://pypi.org/project/pytest-pep8/>`_:
  a ``--pep8`` option to enable PEP8 compliance checking.

* `pytest-flakes <https://pypi.org/project/pytest-flakes/>`_:
  check source code with pyflakes.

* `oejskit <https://pypi.org/project/oejskit/>`_:
  a plugin to run javascript unittests in live browsers.

To see a complete list of all plugins with their latest testing
status against different pytest and Python versions, please visit
`plugincompat <http://plugincompat.herokuapp.com/>`_.

You may also discover more plugins through a `pytest- pypi.org search`_.

.. _`pytest- pypi.org search`: https://pypi.org/search/?q=pytest-


.. _`available installable plugins`:

Requiring/Loading plugins in a test module or conftest file
-----------------------------------------------------------

You can require plugins in a test module or a conftest file like this:

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
