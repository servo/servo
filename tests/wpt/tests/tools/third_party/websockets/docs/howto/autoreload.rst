Reload on code changes
======================

When developing a websockets server, you may run it locally to test changes.
Unfortunately, whenever you want to try a new version of the code, you must
stop the server and restart it, which slows down your development process.

Web frameworks such as Django or Flask provide a development server that
reloads the application automatically when you make code changes. There is no
such functionality in websockets because it's designed for production rather
than development.

However, you can achieve the same result easily.

Install watchdog_ with the ``watchmedo`` shell utility:

.. code-block:: console

    $ pip install 'watchdog[watchmedo]'

.. _watchdog: https://pypi.org/project/watchdog/

Run your server with ``watchmedo auto-restart``:

.. code-block:: console

    $ watchmedo auto-restart --pattern "*.py" --recursive --signal SIGTERM \
        python app.py

This example assumes that the server is defined in a script called ``app.py``.
Adapt it as necessary.
