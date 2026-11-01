Certifi: Python SSL Certificates
================================

Certifi provides Mozilla's carefully curated collection of Root Certificates for
validating the trustworthiness of SSL certificates while verifying the identity
of TLS hosts. It has been extracted from the `Requests`_ project.

Installation
------------

``certifi`` is available on PyPI. Simply install it with ``pip``::

    $ pip install certifi

Usage
-----

To reference the installed certificate authority (CA) bundle, you can use the
built-in function::

    >>> import certifi

    >>> certifi.where()
    '/usr/local/lib/python3.7/site-packages/certifi/cacert.pem'

Or from the command line::

    $ python -m certifi
    /usr/local/lib/python3.7/site-packages/certifi/cacert.pem

Enjoy!

.. _`Requests`: https://requests.readthedocs.io/en/latest/

Addition/Removal of Certificates
--------------------------------

Certifi does not support any addition/removal or other modification of the
CA trust store content. This project is intended to provide a reliable and
highly portable root of trust to python deployments. Look to upstream projects
for methods to use alternate trust.
