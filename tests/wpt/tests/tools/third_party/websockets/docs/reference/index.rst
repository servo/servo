API reference
=============

.. currentmodule:: websockets

Features
--------

Check which implementations support which features and known limitations.

.. toctree::
   :titlesonly:

   features


:mod:`asyncio`
--------------

This is the default implementation. It's ideal for servers that handle many
clients concurrently.

.. toctree::
   :titlesonly:

   asyncio/server
   asyncio/client

:mod:`threading`
----------------

This alternative implementation can be a good choice for clients.

.. toctree::
   :titlesonly:

   sync/server
   sync/client

`Sans-I/O`_
-----------

This layer is designed for integrating in third-party libraries, typically
application servers.

.. _Sans-I/O: https://sans-io.readthedocs.io/

.. toctree::
   :titlesonly:

   sansio/server
   sansio/client

Extensions
----------

The Per-Message Deflate extension is built in. You may also define custom
extensions.

.. toctree::
   :titlesonly:

   extensions

Shared
------

These low-level APIs are shared by all implementations.

.. toctree::
   :titlesonly:

   datastructures
   exceptions
   types

API stability
-------------

Public APIs documented in this API reference are subject to the
:ref:`backwards-compatibility policy <backwards-compatibility policy>`.

Anything that isn't listed in the API reference is a private API. There's no
guarantees of behavior or backwards-compatibility for private APIs.

Convenience imports
-------------------

For convenience, many public APIs can be imported directly from the
``websockets`` package.
