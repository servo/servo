pytest_oejskit plugin (EXTERNAL)
==========================================

The `oejskit`_ offers a ``pytest`` plugin for running Javascript tests in live browsers.   Running inside the browsers comes with some speed cost, on the other hand it means for example the code is tested against the real-word DOM implementations.
The approach enables to write integration tests such that the JavaScript code is tested against server-side Python code mocked as necessary. Any server-side framework that can already be exposed through WSGI (or for which a subset of WSGI can be written to accommodate the jskit own needs) can play along.

For more info and download please visit the `oejskit PyPI`_ page.

.. _`oejskit`:
.. _`oejskit PyPI`: http://pypi.python.org/pypi/oejskit

.. source link 'http://bitbucket.org/pedronis/js-infrastructure/src/tip/pytest_jstests.py',
