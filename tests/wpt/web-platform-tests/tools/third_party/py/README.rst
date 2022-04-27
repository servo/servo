.. image:: https://img.shields.io/pypi/v/py.svg
    :target: https://pypi.org/project/py

.. image:: https://img.shields.io/conda/vn/conda-forge/py.svg
    :target: https://anaconda.org/conda-forge/py

.. image:: https://img.shields.io/pypi/pyversions/py.svg
  :target: https://pypi.org/project/py

.. image:: https://github.com/pytest-dev/py/workflows/build/badge.svg
  :target: https://github.com/pytest-dev/py/actions


**NOTE**: this library is in **maintenance mode** and should not be used in new code.

The py lib is a Python development support library featuring
the following tools and modules:

* ``py.path``:  uniform local and svn path objects  -> please use pathlib/pathlib2 instead
* ``py.apipkg``:  explicit API control and lazy-importing -> please use the standalone package instead
* ``py.iniconfig``:  easy parsing of .ini files -> please use the standalone package instead
* ``py.code``: dynamic code generation and introspection (deprecated, moved to ``pytest`` as a implementation detail).

**NOTE**: prior to the 1.4 release this distribution used to
contain py.test which is now its own package, see https://docs.pytest.org

For questions and more information please visit https://py.readthedocs.io

Bugs and issues: https://github.com/pytest-dev/py

Authors: Holger Krekel and others, 2004-2017
