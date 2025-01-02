Release History
===============

4.0.0 (2020-08-30)
------------------

API Changes (Backward-Incompatible)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Support for Python 2.7 has been removed.
- Support for Python 3.4 has been removed.
- Support for Python 3.5 has been removed.
- Support for PyPy (Python 2.7 compatible) has been removed.
- Support for Python 3.8 has been added.

**Bugfixes**

- Performance improvement of static header search. Use dict search instead
  of linear search.
- Fix debug output of headers during encoding.


3.0.0 (2017-03-29)
------------------

**API Changes (Backward Incompatible)**

- Removed nghttp2 support. This support had rotted and was essentially
  non-functional, so it has now been removed until someone has time to re-add
  the support in a functional form.
- Attempts by the encoder to exceed the maximum allowed header table size via
  dynamic table size updates (or the absence thereof) are now forbidden.

**API Changes (Backward Compatible)**

- Added a new ``InvalidTableSizeError`` thrown when the encoder does not
  respect the maximum table size set by the user.
- Added a ``Decoder.max_allowed_table_size`` field that sets the maximum
  allowed size of the decoder header table. See the documentation for an
  indication of how this should be used.

**Bugfixes**

- Up to 25% performance improvement decoding HPACK-packed integers, depending
  on the platform.
- HPACK now tolerates receiving multiple header table size changes in sequence,
  rather than only one.
- HPACK now forbids header table size changes anywhere but first in a header
  block, as required by RFC 7541 § 4.2.
- Other miscellaneous performance improvements.

2.3.0 (2016-08-04)
------------------

**Security Fixes**

- CVE-2016-6581: HPACK Bomb. This release now enforces a maximum value of the
  decompressed size of the header list. This is to avoid the so-called "HPACK
  Bomb" vulnerability, which is caused when a malicious peer sends a compressed
  HPACK body that decompresses to a gigantic header list size.

  This also adds a ``OversizedHeaderListError``, which is thrown by the
  ``decode`` method if the maximum header list size is being violated. This
  places the HPACK decoder into a broken state: it must not be used after this
  exception is thrown.

  This also adds a ``max_header_list_size`` to the ``Decoder`` object. This
  controls the maximum allowable decompressed size of the header list. By
  default this is set to 64kB.

2.2.0 (2016-04-20)
------------------

**API Changes (Backward Compatible)**

- Added ``HeaderTuple`` and ``NeverIndexedHeaderTuple`` classes that signal
  whether a given header field may ever be indexed in HTTP/2 header
  compression.
- Changed ``Decoder.decode()`` to return the newly added ``HeaderTuple`` class
  and subclass. These objects behave like two-tuples, so this change does not
  break working code.

**Bugfixes**

- Improve Huffman decoding speed by 4x using an approach borrowed from nghttp2.
- Improve HPACK decoding speed by 10% by caching header table sizes.

2.1.1 (2016-03-16)
------------------

**Bugfixes**

- When passing a dictionary or dictionary subclass to ``Encoder.encode``, HPACK
  now ensures that HTTP/2 special headers (headers whose names begin with
  ``:`` characters) appear first in the header block.

2.1.0 (2016-02-02)
------------------

**API Changes (Backward Compatible)**

- Added new ``InvalidTableIndex`` exception, a subclass of
  ``HPACKDecodingError``.
- Instead of throwing ``IndexError`` when encountering invalid encoded integers
  HPACK now throws ``HPACKDecodingError``.
- Instead of throwing ``UnicodeDecodeError`` when encountering headers that are
  not UTF-8 encoded, HPACK now throws ``HPACKDecodingError``.
- Instead of throwing ``IndexError`` when encountering invalid table offsets,
  HPACK now throws ``InvalidTableIndex``.
- Added ``raw`` flag to ``decode``, allowing ``decode`` to return bytes instead
  of attempting to decode the headers as UTF-8.

**Bugfixes**

- ``memoryview`` objects are now used when decoding HPACK, improving the
  performance by avoiding unnecessary data copies.

2.0.1 (2015-11-09)
------------------

- Fixed a bug where the Python HPACK implementation would only emit header
  table size changes for the total change between one header block and another,
  rather than for the entire sequence of changes.

2.0.0 (2015-10-12)
------------------

- Remove unused ``HPACKEncodingError``.
- Add the shortcut ability to import the public API (``Encoder``, ``Decoder``,
  ``HPACKError``, ``HPACKDecodingError``) directly, rather than from
  ``hpack.hpack``.

1.1.0 (2015-07-07)
------------------

- Add support for emitting 'never indexed' header fields, by using an optional
  third element in the header tuple. With thanks to @jimcarreer!

1.0.1 (2015-04-19)
------------------

- Header fields that have names matching header table entries are now added to
  the header table. This improves compression efficiency at the cost of
  slightly more table operations. With thanks to `Tatsuhiro Tsujikawa`_.

.. _Tatsuhiro Tsujikawa: https://github.com/tatsuhiro-t

1.0.0 (2015-04-13)
------------------

- Initial fork of the code from `hyper`_.

.. _hyper: https://hyper.readthedocs.org/
