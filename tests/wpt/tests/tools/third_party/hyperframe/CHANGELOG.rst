Release History
===============

6.0.1 (2021-04-17)
------------------

**API Changes (Backward-compatible)**

- Added support for Python 3.9.
- Added type hints.

6.0.0 (2020-09-06)
------------------

**API Changes (Backward-incompatible)**

- Introduce ``HyperframeError`` base exception class for all errors raised within hyperframe.
- Change exception base class of ``UnknownFrameError`` to ``HyperframeError``
- Change exception base class of ``InvalidPaddingError`` to ``HyperframeError``
- Change exception base class of ``InvalidFrameError`` to ``HyperframeError``
- Invalid frames with wrong stream id (zero vs. non-zero) now raise ``InvalidDataError``.
- Invalid SETTINGS frames (non-empty but ACK) now raise ``InvalidDataError``.
- Invalid ALTSVC frames with non-bytestring field or origin now raise ``InvalidDataError``.

**API Changes (Backward-compatible)**

- Deprecate ``total_padding`` - use `pad_length` instead.
- Improve repr() output for all frame classes.
- Introduce Frame.explain(data) for quick introspection of raw data.

**Bugfixes**

- Fixed padding parsing for ``PushPromiseFrame``.
- Fixed unchecked frame length for ``PriorityFrame``. It now correctly raises ``InvalidFrameError``.
- Fixed promised stream id validation for ``PushPromiseFrame``. It now raises ``InvalidDataError``.
- Fixed unchecked frame length for ``WindowUpdateFrame``. It now correctly raises ``InvalidFrameError``.
- Fixed window increment value range validation. It now raises ``InvalidDataError``.
- Fixed parsing of ``SettingsFrame`` with mutual exclusion of ACK flag and payload.

**Other Changes**

- Removed support for Python 2.7, 3.4, 3.5, pypy.
- Added support for Python 3.8.

5.2.0 (2019-01-18)
------------------

**API Changes (Backward-compatible)**

- Add a new ENABLE_CONNECT_PROTOCOL settings parameter.

**Other Changes**

- Fix collections.abc deprecation.
- Drop support for Python 3.3 and support 3.7.

5.1.0 (2017-04-24)
------------------

**API Changes (Backward-compatible)**

- Added support for ``DataFrame.data`` being a ``memoryview`` object.

5.0.0 (2017-03-07)
------------------

**Backwards Incompatible API Changes**

- Added support for unknown extension frames. These will be returned in the new
  ``ExtensionFrame`` object. The flag information for these frames is persisted
  in ``flag_byte`` if needed.

4.0.2 (2017-02-20)
------------------

**Bugfixes**

- Fixed AltSvc stream association, which was incorrectly set to ``'both'``:
  should have been ``'either'``.
- Fixed a bug where stream IDs on received frames were allowed to be 32-bit,
  instead of 31-bit.
- Fixed a bug with frames that had the ``PADDING`` flag set but zero-length
  padding, whose flow-controlled length was calculated wrongly.
- Miscellaneous performance improvements to serialization and parsing logic.

4.0.1 (2016-03-13)
------------------

**Bugfixes**

- Fixed bug with the repr of ``AltSvcFrame``, where building it could throw
  exceptions if the frame had been received from the network.

4.0.0 (2016-03-13)
------------------

**Backwards Incompatible API Changes**

- Updated old ALTSVC frame definition to match the newly specified RFC 7838.
- Remove BLOCKED frame, which was never actually specified.
- Removed previously deprecated ``SettingsFrame.SETTINGS_MAX_FRAME_SIZE`` and
  ``SettingsFrame.SETTINGS_MAX_HEADER_LIST_SIZE``.

3.2.0 (2016-02-02)
------------------

**API Changes (Backward-compatible)**

- Invalid PING frame bodies now raise ``InvalidFrameError``, not
  ``ValueError``. Note that ``InvalidFrameError`` is a ``ValueError`` subclass.
- Invalid RST_STREAM frame bodies now raise ``InvalidFramError``, not
  ``ValueError``. Note that ``InvalidFrameError`` is a ``ValueError`` subclass.
- Canonicalized the names of ``SettingsFrame.SETTINGS_MAX_FRAME_SIZE`` and
  ``SettingsFrame.SETTINGS_MAX_HEADER_LIST_SIZE`` to match their peers, by
  adding new properties ``SettingsFrame.MAX_FRAME_SIZE`` and
  ``SettingsFrame.SETTINGS_MAX_HEADER_LIST_SIZE``. The old names are still
  present, but will be deprecated in 4.0.0.

**Bugfixes**

- The change in ``3.1.0`` that ensured that ``InvalidFrameError`` would be
  thrown did not affect certain invalid values in ALT_SVC frames. This has been
  fixed: ``ValueError`` will no longer be thrown from invalid ALT_SVC bodies.

3.1.1 (2016-01-18)
------------------

**Bugfixes**

- Correctly error when receiving Ping frames that have insufficient data.

3.1.0 (2016-01-13)
------------------

**API Changes**

- Added new ``InvalidFrameError`` that is thrown instead of ``struct.error``
  when parsing a frame.

**Bugfixes**

- Fixed error when trying to serialize frames that use Priority information
  with the defaults for that information.
- Fixed errors when displaying the repr of frames with non-printable bodies.

3.0.1 (2016-01-08)
------------------

**Bugfixes**

- Fix issue where unpadded DATA, PUSH_PROMISE and HEADERS frames that had empty
  bodies would raise ``InvalidPaddingError`` exceptions when parsed.

3.0.0 (2016-01-08)
------------------

**Backwards Incompatible API Changes**

- Parsing padded frames that have invalid padding sizes now throws an
  ``InvalidPaddingError``.

2.2.0 (2015-10-15)
------------------

**API Changes**

- When an unknown frame is encountered, ``parse_frame_header`` now throws a
  ``ValueError`` subclass: ``UnknownFrameError``. This subclass contains the
  frame type and the length of the frame body.

2.1.0 (2015-10-06)
------------------

**API Changes**

- Frames parsed from binary data now carry a ``body_len`` attribute that
  matches the frame length (minus the frame header).

2.0.0 (2015-09-21)
------------------

**API Changes**

- Attempting to parse unrecognised frames now throws ``ValueError`` instead of
  ``KeyError``.  Thanks to @Kriechi!
- Flags are now validated for correctness, preventing setting flags that
  ``hyperframe`` does not recognise and that would not serialize. Thanks to
  @mhils!
- Frame properties can now be initialized in the constructors. Thanks to @mhils
  and @Kriechi!
- Frames that cannot be sent on a stream now have their stream ID defaulted
  to ``0``. Thanks to @Kriechi!

**Other Changes**

- Frames have a more useful repr. Thanks to @mhils!

1.1.1 (2015-07-20)
------------------

- Fix a bug where ``FRAME_MAX_LEN`` was one byte too small.

1.1.0 (2015-06-28)
------------------

- Add ``body_len`` property to frames to enable introspection of the actual
  frame length. Thanks to @jdecuyper!

1.0.1 (2015-06-27)
------------------

- Fix bug where the frame header would have an incorrect length added to it.

1.0.0 (2015-04-12)
------------------

- Initial extraction from hyper.
