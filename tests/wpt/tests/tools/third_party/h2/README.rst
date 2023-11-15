=========================
h2: HTTP/2 Protocol Stack
=========================

.. image:: https://github.com/python-hyper/h2/workflows/CI/badge.svg
    :target: https://github.com/python-hyper/h2/actions
    :alt: Build Status
.. image:: https://codecov.io/gh/python-hyper/h2/branch/master/graph/badge.svg
    :target: https://codecov.io/gh/python-hyper/h2
    :alt: Code Coverage
.. image:: https://readthedocs.org/projects/h2/badge/?version=latest
    :target: https://h2.readthedocs.io/en/latest/
    :alt: Documentation Status
.. image:: https://img.shields.io/badge/chat-join_now-brightgreen.svg
    :target: https://gitter.im/python-hyper/community
    :alt: Chat community

.. image:: https://raw.github.com/python-hyper/documentation/master/source/logo/hyper-black-bg-white.png

This repository contains a pure-Python implementation of a HTTP/2 protocol
stack. It's written from the ground up to be embeddable in whatever program you
choose to use, ensuring that you can speak HTTP/2 regardless of your
programming paradigm.

You use it like this:

.. code-block:: python

    import h2.connection
    import h2.config

    config = h2.config.H2Configuration()
    conn = h2.connection.H2Connection(config=config)
    conn.send_headers(stream_id=stream_id, headers=headers)
    conn.send_data(stream_id, data)
    socket.sendall(conn.data_to_send())
    events = conn.receive_data(socket_data)

This repository does not provide a parsing layer, a network layer, or any rules
about concurrency. Instead, it's a purely in-memory solution, defined in terms
of data actions and HTTP/2 frames. This is one building block of a full Python
HTTP implementation.

To install it, just run:

.. code-block:: console

    $ python -m pip install h2

Documentation
=============

Documentation is available at https://h2.readthedocs.io .

Contributing
============

``h2`` welcomes contributions from anyone! Unlike many other projects we
are happy to accept cosmetic contributions and small contributions, in addition
to large feature requests and changes.

Before you contribute (either by opening an issue or filing a pull request),
please `read the contribution guidelines`_.

.. _read the contribution guidelines: http://python-hyper.org/en/latest/contributing.html

License
=======

``h2`` is made available under the MIT License. For more details, see the
``LICENSE`` file in the repository.

Authors
=======

``h2`` was authored by Cory Benfield and is maintained
by the members of `python-hyper <https://github.com/orgs/python-hyper/people>`_.
