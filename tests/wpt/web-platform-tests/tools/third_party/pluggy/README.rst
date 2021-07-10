====================================================
pluggy - A minimalist production ready plugin system
====================================================

|pypi| |conda-forge| |versions| |travis| |appveyor| |gitter| |black| |codecov|

This is the core framework used by the `pytest`_, `tox`_, and `devpi`_ projects.

Please `read the docs`_ to learn more!

A definitive example
====================
.. code-block:: python

    import pluggy

    hookspec = pluggy.HookspecMarker("myproject")
    hookimpl = pluggy.HookimplMarker("myproject")


    class MySpec(object):
        """A hook specification namespace.
        """

        @hookspec
        def myhook(self, arg1, arg2):
            """My special little hook that you can customize.
            """


    class Plugin_1(object):
        """A hook implementation namespace.
        """

        @hookimpl
        def myhook(self, arg1, arg2):
            print("inside Plugin_1.myhook()")
            return arg1 + arg2


    class Plugin_2(object):
        """A 2nd hook implementation namespace.
        """

        @hookimpl
        def myhook(self, arg1, arg2):
            print("inside Plugin_2.myhook()")
            return arg1 - arg2


    # create a manager and add the spec
    pm = pluggy.PluginManager("myproject")
    pm.add_hookspecs(MySpec)

    # register plugins
    pm.register(Plugin_1())
    pm.register(Plugin_2())

    # call our ``myhook`` hook
    results = pm.hook.myhook(arg1=1, arg2=2)
    print(results)


Running this directly gets us::

    $ python docs/examples/toy-example.py
    inside Plugin_2.myhook()
    inside Plugin_1.myhook()
    [-1, 3]


.. badges

.. |pypi| image:: https://img.shields.io/pypi/v/pluggy.svg
    :target: https://pypi.org/pypi/pluggy

.. |versions| image:: https://img.shields.io/pypi/pyversions/pluggy.svg
    :target: https://pypi.org/pypi/pluggy

.. |travis| image:: https://img.shields.io/travis/pytest-dev/pluggy/master.svg
    :target: https://travis-ci.org/pytest-dev/pluggy

.. |appveyor| image:: https://img.shields.io/appveyor/ci/pytestbot/pluggy/master.svg
    :target: https://ci.appveyor.com/project/pytestbot/pluggy

.. |conda-forge| image:: https://img.shields.io/conda/vn/conda-forge/pluggy.svg
    :target: https://anaconda.org/conda-forge/pytest

.. |gitter| image:: https://badges.gitter.im/pytest-dev/pluggy.svg
    :alt: Join the chat at https://gitter.im/pytest-dev/pluggy
    :target: https://gitter.im/pytest-dev/pluggy?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge

.. |black| image:: https://img.shields.io/badge/code%20style-black-000000.svg
    :target: https://github.com/ambv/black

.. |codecov| image:: https://codecov.io/gh/pytest-dev/pluggy/branch/master/graph/badge.svg
    :target: https://codecov.io/gh/pytest-dev/pluggy
    :alt: Code coverage Status

.. links
.. _pytest:
    http://pytest.org
.. _tox:
    https://tox.readthedocs.org
.. _devpi:
    http://doc.devpi.net
.. _read the docs:
   https://pluggy.readthedocs.io/en/latest/
