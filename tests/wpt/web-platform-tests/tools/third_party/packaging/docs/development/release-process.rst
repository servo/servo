Release Process
===============

#. Checkout the current ``master`` branch.
#. Install the latest ``nox``::

    $ pip install nox

#. Run the release automation with the required version number (YY.N)::

    $ nox -s release -- YY.N

#. Notify the other project owners of the release.

.. note::

   Access needed for making the release are:

   - PyPI maintainer (or owner) access to `packaging`
   - push directly to the `master` branch on the source repository
   - push tags directly to the source repository
