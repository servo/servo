Release History
===============

3.0.1 (2017-04-03)
------------------

Bugfixes
~~~~~~~~

- CONTINUATION frames sent on closed streams previously caused stream errors
  of type STREAM_CLOSED. RFC 7540 § 6.10 requires that these be connection
  errors of type PROTOCOL_ERROR, and so this release changes to match that
  behaviour.
- Remote peers incrementing their inbound connection window beyond the maximum
  allowed value now cause stream-level errors, rather than connection-level
  errors, allowing connections to stay up longer.
- h2 now rejects receiving and sending request header blocks that are missing
  any of the mandatory pseudo-header fields (:path, :scheme, and :method).
- h2 now rejects receiving and sending request header blocks that have an empty
  :path pseudo-header value.
- h2 now rejects receiving and sending request header blocks that contain
  response-only pseudo-headers, and vice versa.
- h2 now correct respects user-initiated changes to the HEADER_TABLE_SIZE
  local setting, and ensures that if users shrink or increase the header
  table size it is policed appropriately.


2.6.2 (2017-04-03)
------------------

Bugfixes
~~~~~~~~

- CONTINUATION frames sent on closed streams previously caused stream errors
  of type STREAM_CLOSED. RFC 7540 § 6.10 requires that these be connection
  errors of type PROTOCOL_ERROR, and so this release changes to match that
  behaviour.
- Remote peers incrementing their inbound connection window beyond the maximum
  allowed value now cause stream-level errors, rather than connection-level
  errors, allowing connections to stay up longer.
- h2 now rejects receiving and sending request header blocks that are missing
  any of the mandatory pseudo-header fields (:path, :scheme, and :method).
- h2 now rejects receiving and sending request header blocks that have an empty
  :path pseudo-header value.
- h2 now rejects receiving and sending request header blocks that contain
  response-only pseudo-headers, and vice versa.
- h2 now correct respects user-initiated changes to the HEADER_TABLE_SIZE
  local setting, and ensures that if users shrink or increase the header
  table size it is policed appropriately.


2.5.4 (2017-04-03)
------------------

Bugfixes
~~~~~~~~

- CONTINUATION frames sent on closed streams previously caused stream errors
  of type STREAM_CLOSED. RFC 7540 § 6.10 requires that these be connection
  errors of type PROTOCOL_ERROR, and so this release changes to match that
  behaviour.
- Remote peers incrementing their inbound connection window beyond the maximum
  allowed value now cause stream-level errors, rather than connection-level
  errors, allowing connections to stay up longer.
- h2 now correct respects user-initiated changes to the HEADER_TABLE_SIZE
  local setting, and ensures that if users shrink or increase the header
  table size it is policed appropriately.


3.0.0 (2017-03-24)
------------------

API Changes (Backward-Incompatible)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- By default, hyper-h2 now joins together received cookie header fields, per
  RFC 7540 Section 8.1.2.5.
- Added a ``normalize_inbound_headers`` flag to the ``H2Configuration`` object
  that defaults to ``True``. Setting this to ``False`` changes the behaviour
  from the previous point back to the v2 behaviour.
- Removed deprecated fields from ``h2.errors`` module.
- Removed deprecated fields from ``h2.settings`` module.
- Removed deprecated ``client_side`` and ``header_encoding`` arguments from
  ``H2Connection``.
- Removed deprecated ``client_side`` and ``header_encoding`` properties from
  ``H2Connection``.
- ``dict`` objects are no longer allowed for user-supplied headers.
- The default header encoding is now ``None``, not ``utf-8``: this means that
  all events that carry headers now return those headers as byte strings by
  default. The header encoding can be set back to ``utf-8`` to restore the old
  behaviour.

API Changes (Backward-Compatible)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Added new ``UnknownFrameReceived`` event that fires when unknown extension
  frames have been received. This only fires when using hyperframe 5.0 or
  later: earlier versions of hyperframe cause us to silently ignore extension
  frames.

Bugfixes
~~~~~~~~

None


2.6.1 (2017-03-16)
------------------

Bugfixes
~~~~~~~~

- Allowed hyperframe v5 support while continuing to ignore unexpected frames.


2.5.3 (2017-03-16)
------------------

Bugfixes
~~~~~~~~

- Allowed hyperframe v5 support while continuing to ignore unexpected frames.


2.4.4 (2017-03-16)
------------------

Bugfixes
~~~~~~~~

- Allowed hyperframe v5 support while continuing to ignore unexpected frames.


2.6.0 (2017-02-28)
------------------

API Changes (Backward-Compatible)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Added a new ``h2.events.Event`` class that acts as a base class for all
  events.
- Rather than reject outbound Connection-specific headers, h2 will now
  normalize the header block by removing them.
- Implement equality for the ``h2.settings.Settings`` class.
- Added ``h2.settings.SettingCodes``, an enum that is used to store all the
  HTTP/2 setting codes. This allows us to use a better printed representation of
  the setting code in most places that it is used.
- The ``setting`` field in ``ChangedSetting`` for the ``RemoteSettingsChanged``
  and ``SettingsAcknowledged`` events has been updated to be instances of
  ``SettingCodes`` whenever they correspond to a known setting code. When they
  are an unknown setting code, they are instead ``int``. As ``SettingCodes`` is
  a subclass of ``int``, this is non-breaking.
- Deprecated the other fields in ``h2.settings``. These will be removed in
  3.0.0.
- Added an optional ``pad_length`` parameter to ``H2Connection.send_data``
  to allow the user to include padding on a data frame.
- Added a new parameter to the ``h2.config.H2Configuration`` initializer which
  takes a logger.  This allows us to log by providing a logger that conforms
  to the requirements of this module so that it can be used in different
  environments.

Bugfixes
~~~~~~~~

- Correctly reject pushed request header blocks whenever they have malformed
  request header blocks.
- Correctly normalize pushed request header blocks whenever they have
  normalizable header fields.
- Remote peers are now allowed to send zero or any positive number as a value
  for ``SETTINGS_MAX_HEADER_LIST_SIZE``, where previously sending zero would
  raise a ``InvalidSettingsValueError``.
- Resolved issue where the ``HTTP2-Settings`` header value for plaintext
  upgrade that was emitted by ``initiate_upgrade_connection`` included the
  *entire* ``SETTINGS`` frame, instead of just the payload.
- Resolved issue where the ``HTTP2-Settings`` header value sent by a client for
  plaintext upgrade would be ignored by ``initiate_upgrade_connection``, rather
  than have those settings applied appropriately.
- Resolved an issue whereby certain frames received from a peer in the CLOSED
  state would trigger connection errors when RFC 7540 says they should have
  triggered stream errors instead. Added more detailed stream closure tracking
  to ensure we don't throw away connections unnecessarily.


2.5.2 (2017-01-27)
------------------

- Resolved issue where the ``HTTP2-Settings`` header value for plaintext
  upgrade that was emitted by ``initiate_upgrade_connection`` included the
  *entire* ``SETTINGS`` frame, instead of just the payload.
- Resolved issue where the ``HTTP2-Settings`` header value sent by a client for
  plaintext upgrade would be ignored by ``initiate_upgrade_connection``, rather
  than have those settings applied appropriately.


2.4.3 (2017-01-27)
------------------

- Resolved issue where the ``HTTP2-Settings`` header value for plaintext
  upgrade that was emitted by ``initiate_upgrade_connection`` included the
  *entire* ``SETTINGS`` frame, instead of just the payload.
- Resolved issue where the ``HTTP2-Settings`` header value sent by a client for
  plaintext upgrade would be ignored by ``initiate_upgrade_connection``, rather
  than have those settings applied appropriately.


2.3.4 (2017-01-27)
------------------

- Resolved issue where the ``HTTP2-Settings`` header value for plaintext
  upgrade that was emitted by ``initiate_upgrade_connection`` included the
  *entire* ``SETTINGS`` frame, instead of just the payload.
- Resolved issue where the ``HTTP2-Settings`` header value sent by a client for
  plaintext upgrade would be ignored by ``initiate_upgrade_connection``, rather
  than have those settings applied appropriately.


2.5.1 (2016-12-17)
------------------

Bugfixes
~~~~~~~~

- Remote peers are now allowed to send zero or any positive number as a value
  for ``SETTINGS_MAX_HEADER_LIST_SIZE``, where previously sending zero would
  raise a ``InvalidSettingsValueError``.


2.5.0 (2016-10-25)
------------------

API Changes (Backward-Compatible)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Added a new ``H2Configuration`` object that allows rich configuration of
  a ``H2Connection``. This object supersedes the prior keyword arguments to the
  ``H2Connection`` object, which are now deprecated and will be removed in 3.0.
- Added support for automated window management via the
  ``acknowledge_received_data`` method. See the documentation for more details.
- Added a ``DenialOfServiceError`` that is raised whenever a behaviour that
  looks like a DoS attempt is encountered: for example, an overly large
  decompressed header list. This is a subclass of ``ProtocolError``.
- Added support for setting and managing ``SETTINGS_MAX_HEADER_LIST_SIZE``.
  This setting is now defaulted to 64kB.
- Added ``h2.errors.ErrorCodes``, an enum that is used to store all the HTTP/2
  error codes. This allows us to use a better printed representation of the
  error code in most places that it is used.
- The ``error_code`` fields on ``ConnectionTerminated`` and ``StreamReset``
  events have been updated to be instances of ``ErrorCodes`` whenever they
  correspond to a known error code. When they are an unknown error code, they
  are instead ``int``. As ``ErrorCodes`` is a subclass of ``int``, this is
  non-breaking.
- Deprecated the other fields in ``h2.errors``. These will be removed in 3.0.0.

Bugfixes
~~~~~~~~

- Correctly reject request header blocks with neither :authority nor Host
  headers, or header blocks which contain mismatched :authority and Host
  headers, per RFC 7540 Section 8.1.2.3.
- Correctly expect that responses to HEAD requests will have no body regardless
  of the value of the Content-Length header, and reject those that do.
- Correctly refuse to send header blocks that contain neither :authority nor
  Host headers, or header blocks which contain mismatched :authority and Host
  headers, per RFC 7540 Section 8.1.2.3.
- Hyper-h2 will now reject header field names and values that contain leading
  or trailing whitespace.
- Correctly strip leading/trailing whitespace from header field names and
  values.
- Correctly refuse to send header blocks with a TE header whose value is not
  ``trailers``, per RFC 7540 Section 8.1.2.2.
- Correctly refuse to send header blocks with connection-specific headers,
  per RFC 7540 Section 8.1.2.2.
- Correctly refuse to send header blocks that contain duplicate pseudo-header
  fields, or with pseudo-header fields that appear after ordinary header fields,
  per RFC 7540 Section 8.1.2.1.

  This may cause passing a dictionary as the header block to ``send_headers``
  to throw a ``ProtocolError``, because dictionaries are unordered and so they
  may trip this check.  Passing dictionaries here is deprecated, and callers
  should change to using a sequence of 2-tuples as their header blocks.
- Correctly reject trailers that contain HTTP/2 pseudo-header fields, per RFC
  7540 Section 8.1.2.1.
- Correctly refuse to send trailers that contain HTTP/2 pseudo-header fields,
  per RFC 7540 Section 8.1.2.1.
- Correctly reject responses that do not contain the ``:status`` header field,
  per RFC 7540 Section 8.1.2.4.
- Correctly refuse to send responses that do not contain the ``:status`` header
  field, per RFC 7540 Section 8.1.2.4.
- Correctly update the maximum frame size when the user updates the value of
  that setting. Prior to this release, if the user updated the maximum frame
  size hyper-h2 would ignore the update, preventing the remote peer from using
  the higher frame sizes.

2.4.2 (2016-10-25)
------------------

Bugfixes
~~~~~~~~

- Correctly update the maximum frame size when the user updates the value of
  that setting. Prior to this release, if the user updated the maximum frame
  size hyper-h2 would ignore the update, preventing the remote peer from using
  the higher frame sizes.

2.3.3 (2016-10-25)
------------------

Bugfixes
~~~~~~~~

- Correctly update the maximum frame size when the user updates the value of
  that setting. Prior to this release, if the user updated the maximum frame
  size hyper-h2 would ignore the update, preventing the remote peer from using
  the higher frame sizes.

2.2.7 (2016-10-25)
------------------

*Final 2.2.X release*

Bugfixes
~~~~~~~~

- Correctly update the maximum frame size when the user updates the value of
  that setting. Prior to this release, if the user updated the maximum frame
  size hyper-h2 would ignore the update, preventing the remote peer from using
  the higher frame sizes.

2.4.1 (2016-08-23)
------------------

Bugfixes
~~~~~~~~

- Correctly expect that responses to HEAD requests will have no body regardless
  of the value of the Content-Length header, and reject those that do.

2.3.2 (2016-08-23)
------------------

Bugfixes
~~~~~~~~

- Correctly expect that responses to HEAD requests will have no body regardless
  of the value of the Content-Length header, and reject those that do.

2.4.0 (2016-07-01)
------------------

API Changes (Backward-Compatible)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Adds ``additional_data`` to ``H2Connection.close_connection``, allowing the
  user to send additional debug data on the GOAWAY frame.
- Adds ``last_stream_id`` to ``H2Connection.close_connection``, allowing the
  user to manually control what the reported last stream ID is.
- Add new method: ``prioritize``.
- Add support for emitting stream priority information when sending headers
  frames using three new keyword arguments: ``priority_weight``,
  ``priority_depends_on``, and ``priority_exclusive``.
- Add support for "related events": events that fire simultaneously on a single
  frame.


2.3.1 (2016-05-12)
------------------

Bugfixes
~~~~~~~~

- Resolved ``AttributeError`` encountered when receiving more than one sequence
  of CONTINUATION frames on a given connection.


2.2.5 (2016-05-12)
------------------

Bugfixes
~~~~~~~~

- Resolved ``AttributeError`` encountered when receiving more than one sequence
  of CONTINUATION frames on a given connection.


2.3.0 (2016-04-26)
------------------

API Changes (Backward-Compatible)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Added a new flag to the ``H2Connection`` constructor: ``header_encoding``,
  that controls what encoding is used (if any) to decode the headers from bytes
  to unicode. This defaults to UTF-8 for backward compatibility. To disable the
  decode and use bytes exclusively, set the field to False, None, or the empty
  string. This affects all headers, including those pushed by servers.
- Bumped the minimum version of HPACK allowed from 2.0 to 2.2.
- Added support for advertising RFC 7838 Alternative services.
- Allowed users to provide ``hpack.HeaderTuple`` and
  ``hpack.NeverIndexedHeaderTuple`` objects to all methods that send headers.
- Changed all events that carry headers to emit ``hpack.HeaderTuple`` and
  ``hpack.NeverIndexedHeaderTuple`` instead of plain tuples. This allows users
  to maintain header indexing state.
- Added support for plaintext upgrade with the ``initiate_upgrade_connection``
  method.

Bugfixes
~~~~~~~~

- Automatically ensure that all ``Authorization`` and ``Proxy-Authorization``
  headers, as well as short ``Cookie`` headers, are prevented from being added
  to encoding contexts.

2.2.4 (2016-04-25)
------------------

Bugfixes
~~~~~~~~

- Correctly forbid pseudo-headers that were not defined in RFC 7540.
- Ignore AltSvc frames, rather than exploding when receiving them.

2.1.5 (2016-04-25)
------------------

*Final 2.1.X release*

Bugfixes
~~~~~~~~

- Correctly forbid pseudo-headers that were not defined in RFC 7540.
- Ignore AltSvc frames, rather than exploding when receiving them.

2.2.3 (2016-04-13)
------------------

Bugfixes
~~~~~~~~

- Allowed the 4.X series of hyperframe releases as dependencies.

2.1.4 (2016-04-13)
------------------

Bugfixes
~~~~~~~~

- Allowed the 4.X series of hyperframe releases as dependencies.


2.2.2 (2016-04-05)
------------------

Bugfixes
~~~~~~~~

- Fixed issue where informational responses were erroneously not allowed to be
  sent in the ``HALF_CLOSED_REMOTE`` state.
- Fixed issue where informational responses were erroneously not allowed to be
  received in the ``HALF_CLOSED_LOCAL`` state.
- Fixed issue where we allowed information responses to be sent or received
  after final responses.

2.2.1 (2016-03-23)
------------------

Bugfixes
~~~~~~~~

- Fixed issue where users using locales that did not default to UTF-8 were
  unable to install source distributions of the package.

2.2.0 (2016-03-23)
------------------

API Changes (Backward-Compatible)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Added support for sending informational responses (responses with 1XX status)
  codes as part of the standard flow. HTTP/2 allows zero or more informational
  responses with no upper limit: hyper-h2 does too.
- Added support for receiving informational responses (responses with 1XX
  status) codes as part of the standard flow. HTTP/2 allows zero or more
  informational responses with no upper limit: hyper-h2 does too.
- Added a new event: ``ReceivedInformationalResponse``. This response is fired
  when informational responses (those with 1XX status codes).
- Added an ``additional_data`` field to the ``ConnectionTerminated`` event that
  carries any additional data sent on the GOAWAY frame. May be ``None`` if no
  such data was sent.
- Added the ``initial_values`` optional argument to the ``Settings`` object.

Bugfixes
~~~~~~~~

- Correctly reject all of the connection-specific headers mentioned in RFC 7540
  § 8.1.2.2, not just the ``Connection:`` header.
- Defaulted the value of ``SETTINGS_MAX_CONCURRENT_STREAMS`` to 100, unless
  explicitly overridden. This is a safe defensive initial value for this
  setting.

2.1.3 (2016-03-16)
------------------

Deprecations
~~~~~~~~~~~~

- Passing dictionaries to ``send_headers`` as the header block is deprecated,
  and will be removed in 3.0.

2.1.2 (2016-02-17)
------------------

Bugfixes
~~~~~~~~

- Reject attempts to push streams on streams that were themselves pushed:
  streams can only be pushed on streams that were initiated by the client.
- Correctly allow CONTINUATION frames to extend the header block started by a
  PUSH_PROMISE frame.
- Changed our handling of frames received on streams that were reset by the
  user.

  Previously these would, at best, cause ProtocolErrors to be raised and the
  connection to be torn down (rather defeating the point of resetting streams
  at all) and, at worst, would cause subtle inconsistencies in state between
  hyper-h2 and the remote peer that could lead to header block decoding errors
  or flow control blockages.

  Now when the user resets a stream all further frames received on that stream
  are ignored except where they affect some form of connection-level state,
  where they have their effect and are then ignored.
- Fixed a bug whereby receiving a PUSH_PROMISE frame on a stream that was
  closed would cause a RST_STREAM frame to be emitted on the closed-stream,
  but not the newly-pushed one. Now this causes a ``ProtocolError``.

2.1.1 (2016-02-05)
------------------

Bugfixes
~~~~~~~~

- Added debug representations for all events.
- Fixed problems with setup.py that caused trouble on older setuptools/pip
  installs.

2.1.0 (2016-02-02)
------------------

API Changes (Backward-Compatible)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Added new field to ``DataReceived``: ``flow_controlled_length``. This is the
  length of the frame including padded data, allowing users to correctly track
  changes to the flow control window.
- Defined new ``UnsupportedFrameError``, thrown when frames that are known to
  hyperframe but not supported by hyper-h2 are received. For
  backward-compatibility reasons, this is a ``ProtocolError`` *and* a
  ``KeyError``.

Bugfixes
~~~~~~~~

- Hyper-h2 now correctly accounts for padding when maintaining flow control
  windows.
- Resolved a bug where hyper-h2 would mistakenly apply
  SETTINGS_INITIAL_WINDOW_SIZE to the connection flow control window in
  addition to the stream-level flow control windows.
- Invalid Content-Length headers now throw ``ProtocolError`` exceptions and
  correctly tear the connection down, instead of leaving the connection in an
  indeterminate state.
- Invalid header blocks now throw ``ProtocolError``, rather than a grab bag of
  possible other exceptions.

2.0.0 (2016-01-25)
------------------

API Changes (Breaking)
~~~~~~~~~~~~~~~~~~~~~~

- Attempts to open streams with invalid stream IDs, either by the remote peer
  or by the user, are now rejected as a ``ProtocolError``. Previously these
  were allowed, and would cause remote peers to error.
- Receiving frames that have invalid padding now causes the connection to be
  terminated with a ``ProtocolError`` being raised. Previously these passed
  undetected.
- Settings values set by both the user and the remote peer are now validated
  when they're set. If they're invalid, a new ``InvalidSettingsValueError`` is
  raised and, if set by the remote peer, a connection error is signaled.
  Previously, it was possible to set invalid values. These would either be
  caught when building frames, or would be allowed to stand.
- Settings changes no longer require user action to be acknowledged: hyper-h2
  acknowledges them automatically. This moves the location where some
  exceptions may be thrown, and also causes the ``acknowledge_settings`` method
  to be removed from the public API.
- Removed a number of methods on the ``H2Connection`` object from the public,
  semantically versioned API, by renaming them to have leading underscores.
  Specifically, removed:

    - ``get_stream_by_id``
    - ``get_or_create_stream``
    - ``begin_new_stream``
    - ``receive_frame``
    - ``acknowledge_settings``

- Added full support for receiving CONTINUATION frames, including policing
  logic about when and how they are received. Previously, receiving
  CONTINUATION frames was not supported and would throw exceptions.
- All public API functions on ``H2Connection`` except for ``receive_data`` no
  longer return lists of events, because these lists were always empty. Events
  are now only raised by ``receive_data``.
- Calls to ``increment_flow_control_window`` with out of range values now raise
  ``ValueError`` exceptions. Previously they would be allowed, or would cause
  errors when serializing frames.

API Changes (Backward-Compatible)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Added ``PriorityUpdated`` event for signaling priority changes.
- Added ``get_next_available_stream_id`` function.
- Receiving DATA frames on streams not in the OPEN or HALF_CLOSED_LOCAL states
  now causes a stream reset, rather than a connection reset. The error is now
  also classified as a ``StreamClosedError``, rather than a more generic
  ``ProtocolError``.
- Receiving HEADERS or PUSH_PROMISE frames in the HALF_CLOSED_REMOTE state now
  causes a stream reset, rather than a connection reset.
- Receiving frames that violate the max frame size now causes connection errors
  with error code FRAME_SIZE_ERROR, not a generic PROTOCOL_ERROR. This
  condition now also raises a ``FrameTooLargeError``, a new subclass of
  ``ProtocolError``.
- Made ``NoSuchStreamError`` a subclass of ``ProtocolError``.
- The ``StreamReset`` event is now also fired whenever a protocol error from
  the remote peer forces a stream to close early. This is only fired once.
- The ``StreamReset`` event now carries a flag, ``remote_reset``, that is set
  to ``True`` in all cases where ``StreamReset`` would previously have fired
  (e.g. when the remote peer sent a RST_STREAM), and is set to ``False`` when
  it fires because the remote peer made a protocol error.
- Hyper-h2 now rejects attempts by peers to increment a flow control window by
  zero bytes.
- Hyper-h2 now rejects peers sending header blocks that are ill-formed for a
  number of reasons as set out in RFC 7540 Section 8.1.2.
- Attempting to send non-PRIORITY frames on closed streams now raises
  ``StreamClosedError``.
- Remote peers attempting to increase the flow control window beyond
  ``2**31 - 1``, either by window increment or by settings frame, are now
  rejected as ``ProtocolError``.
- Local attempts to increase the flow control window beyond ``2**31 - 1`` by
  window increment are now rejected as ``ProtocolError``.
- The bytes that represent individual settings are now available in
  ``h2.settings``, instead of needing users to import them from hyperframe.

Bugfixes
~~~~~~~~

- RFC 7540 requires that a separate minimum stream ID be used for inbound and
  outbound streams. Hyper-h2 now obeys this requirement.
- Hyper-h2 now does a better job of reporting the last stream ID it has
  partially handled when terminating connections.
- Fixed an error in the arguments of ``StreamIDTooLowError``.
- Prevent ``ValueError`` leaking from Hyperframe.
- Prevent ``struct.error`` and ``InvalidFrameError`` leaking from Hyperframe.

1.1.1 (2015-11-17)
------------------

Bugfixes
~~~~~~~~

- Forcibly lowercase all header names to improve compatibility with
  implementations that demand lower-case header names.

1.1.0 (2015-10-28)
------------------

API Changes (Backward-Compatible)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Added a new ``ConnectionTerminated`` event, which fires when GOAWAY frames
  are received.
- Added a subclass of ``NoSuchStreamError``, called ``StreamClosedError``, that
  fires when actions are taken on a stream that is closed and has had its state
  flushed from the system.
- Added ``StreamIDTooLowError``, raised when the user or the remote peer
  attempts to create a stream with an ID lower than one previously used in the
  dialog. Inherits from ``ValueError`` for backward-compatibility reasons.

Bugfixes
~~~~~~~~

- Do not throw ``ProtocolError`` when attempting to send multiple GOAWAY
  frames on one connection.
- We no longer forcefully change the decoder table size when settings changes
  are ACKed, instead waiting for remote acknowledgement of the change.
- Improve the performance of checking whether a stream is open.
- We now attempt to lazily garbage collect closed streams, to avoid having the
  state hang around indefinitely, leaking memory.
- Avoid further per-stream allocations, leading to substantial performance
  improvements when many short-lived streams are used.

1.0.0 (2015-10-15)
------------------

- First production release!
