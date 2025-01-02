hpack: HTTP/2 Header Compression for Python
===========================================

hpack provides a simple Python interface to the `HPACK`_ compression algorithm,
used to compress HTTP headers in HTTP/2. Used by some of the most popular
HTTP/2 implementations in Python, HPACK offers a great Python interface as well
as optional upgrade to optimised C-based compression routines from `nghttp2`_.

Using hpack is easy:

.. code-block:: python

    from hpack import Encoder, Decoder

    e = Encoder()
    encoded_bytes = e.encode(headers)

    d = Decoder()
    decoded_headers = d.decode(encoded_bytes)

hpack will transparently use nghttp2 on CPython if it's available, gaining even
better compression efficiency and speed, but it also makes available a
pure-Python implementation that conforms strictly to `RFC 7541`_.

Contents
--------

.. toctree::
   :maxdepth: 2

   installation
   api
   security/index


.. _HPACK: https://tools.ietf.org/html/rfc7541
.. _nghttp2: https://nghttp2.org/
.. _RFC 7541: https://tools.ietf.org/html/rfc7541
