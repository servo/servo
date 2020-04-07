'use strict';

// Test is based on the following editor draft:
// https://w3c.github.io/webrtc-pc/archives/20170605/webrtc.html

// Helper function for testing RTCRtpParameters dictionary fields

// This file depends on dictionary-helper.js which should
// be loaded from the main HTML file.

// An offer/answer exchange is necessary for getParameters() to have any
// negotiated parameters to return.
async function doOfferAnswerExchange(t, caller) {
  const callee = new RTCPeerConnection();
  t.add_cleanup(() => callee.close());
  const offer = await caller.createOffer();
  await caller.setLocalDescription(offer);
  await callee.setRemoteDescription(offer);
  const answer = await callee.createAnswer();
  await callee.setLocalDescription(answer);
  await caller.setRemoteDescription(answer);

  return callee;
}

/*
  Validates the RTCRtpParameters returned from RTCRtpSender.prototype.getParameters

  5.2.  RTCRtpSender Interface
    getParameters
      - transactionId is set to a new unique identifier, used to match this getParameters
        call to a setParameters call that may occur later.

      - encodings is set to the value of the [[SendEncodings]] internal slot.

      - The headerExtensions sequence is populated based on the header extensions that
        have been negotiated for sending.

      - The codecs sequence is populated based on the codecs that have been negotiated
        for sending, and which the user agent is currently capable of sending. If
        setParameters has removed or reordered codecs, getParameters MUST return the
        shortened/reordered list. However, every time codecs are renegotiated by a
        new offer/answer exchange, the list of codecs MUST be restored to the full
        negotiated set, in the priority order indicated by the remote description,
        in effect discarding the effects of setParameters.

      - rtcp.cname is set to the CNAME of the associated RTCPeerConnection. rtcp.reducedSize
        is set to true if reduced-size RTCP has been negotiated for sending, and false otherwise.
 */
function validateSenderRtpParameters(param) {
  validateRtpParameters(param);

  assert_not_equals(param.transactionId, undefined,
    'Expect sender param.transactionId to be set');

  assert_not_equals(param.rtcp.cname, undefined,
    'Expect sender param.rtcp.cname to be set');

  assert_not_equals(param.rtcp.reducedSize, undefined,
    'Expect sender param.rtcp.reducedSize to be set to either true or false');
}

/*
  Validates the RTCRtpParameters returned from RTCRtpReceiver.prototype.getParameters

  5.3.  RTCRtpReceiver Interface
    getParameters
      When getParameters is called, the RTCRtpParameters dictionary is constructed
      as follows:

      - The headerExtensions sequence is populated based on the header extensions that
        the receiver is currently prepared to receive.

      - The codecs sequence is populated based on the codecs that the receiver is currently
        prepared to receive.

      - rtcp.reducedSize is set to true if the receiver is currently prepared to receive
        reduced-size RTCP packets, and false otherwise. rtcp.cname is left undefined.

      - transactionId is left undefined.
 */
function validateReceiverRtpParameters(param) {
  validateRtpParameters(param);

  assert_equals(param.transactionId, undefined,
    'Expect receiver param.transactionId to be unset');

  assert_not_equals(param.rtcp.reducedSize, undefined,
    'Expect receiver param.rtcp.reducedSize to be set');

  assert_equals(param.rtcp.cname, undefined,
    'Expect receiver param.rtcp.cname to be unset');
}

/*
  dictionary RTCRtpParameters {
    DOMString                                 transactionId;
    sequence<RTCRtpEncodingParameters>        encodings;
    sequence<RTCRtpHeaderExtensionParameters> headerExtensions;
    RTCRtcpParameters                         rtcp;
    sequence<RTCRtpCodecParameters>           codecs;
  };

  enum RTCDegradationPreference {
    "maintain-framerate",
    "maintain-resolution",
    "balanced"
  };
 */
function validateRtpParameters(param) {
  assert_optional_string_field(param, 'transactionId');

  assert_array_field(param, 'encodings');
  for(const encoding of param.encodings) {
    validateEncodingParameters(encoding);
  }

  assert_array_field(param, 'headerExtensions');
  for(const headerExt of param.headerExtensions) {
    validateHeaderExtensionParameters(headerExt);
  }

  assert_dict_field(param, 'rtcp');
  validateRtcpParameters(param.rtcp);

  assert_array_field(param, 'codecs');
  for(const codec of param.codecs) {
    validateCodecParameters(codec);
  }
}

/*
  dictionary RTCRtpEncodingParameters {
    RTCDtxStatus        dtx;
    boolean             active;
    RTCPriorityType     priority;
    RTCPriorityType     networkPriority;
    unsigned long       ptime;
    unsigned long       maxBitrate;
    double              maxFramerate;

    [readonly]
    DOMString           rid;

    double              scaleResolutionDownBy;
  };

  enum RTCDtxStatus {
    "disabled",
    "enabled"
  };

  enum RTCPriorityType {
    "very-low",
    "low",
    "medium",
    "high"
  };
 */
function validateEncodingParameters(encoding) {
  assert_optional_enum_field(encoding, 'dtx',
    ['disabled', 'enabled']);

  assert_optional_boolean_field(encoding, 'active');
  assert_optional_enum_field(encoding, 'priority',
    ['very-low', 'low', 'medium', 'high']);
  assert_optional_enum_field(encoding, 'networkPriority',
    ['very-low', 'low', 'medium', 'high']);

  assert_optional_unsigned_int_field(encoding, 'ptime');
  assert_optional_unsigned_int_field(encoding, 'maxBitrate');
  assert_optional_number_field(encoding, 'maxFramerate');

  assert_optional_string_field(encoding, 'rid');
  assert_optional_number_field(encoding, 'scaleResolutionDownBy');
}

/*
  dictionary RTCRtcpParameters {
    [readonly]
    DOMString cname;

    [readonly]
    boolean   reducedSize;
  };
 */
function validateRtcpParameters(rtcp) {
  assert_optional_string_field(rtcp, 'cname');
  assert_optional_boolean_field(rtcp, 'reducedSize');
}

/*
  dictionary RTCRtpHeaderExtensionParameters {
    [readonly]
    DOMString      uri;

    [readonly]
    unsigned short id;

    [readonly]
    boolean        encrypted;
  };
 */
function validateHeaderExtensionParameters(headerExt) {
  assert_optional_string_field(headerExt, 'uri');
  assert_optional_unsigned_int_field(headerExt, 'id');
  assert_optional_boolean_field(headerExt, 'encrypted');
}

/*
  dictionary RTCRtpCodecParameters {
    [readonly]
    unsigned short payloadType;

    [readonly]
    DOMString      mimeType;

    [readonly]
    unsigned long  clockRate;

    [readonly]
    unsigned short channels;

    [readonly]
    DOMString      sdpFmtpLine;
  };
 */
function validateCodecParameters(codec) {
  assert_optional_unsigned_int_field(codec, 'payloadType');
  assert_optional_string_field(codec, 'mimeType');
  assert_optional_unsigned_int_field(codec, 'clockRate');
  assert_optional_unsigned_int_field(codec, 'channels');
  assert_optional_string_field(codec, 'sdpFmtpLine');
}

// Get the first encoding in param.encodings.
// Asserts that param.encodings has at least one element.
function getFirstEncoding(param) {
  const {
    encodings
  } = param;
  assert_equals(encodings.length, 1);
  return encodings[0];
}

// Helper function to test that modifying an encoding field should succeed
function test_modified_encoding(kind, field, value1, value2, desc) {
  promise_test(async t => {
    const pc = new RTCPeerConnection();
    t.add_cleanup(() => pc.close());
    const {
      sender
    } = pc.addTransceiver(kind, {
      sendEncodings: [{
        [field]: value1
      }]
    });
    await doOfferAnswerExchange(t, pc);

    const param1 = sender.getParameters();
    validateSenderRtpParameters(param1);
    const encoding1 = getFirstEncoding(param1);

    assert_equals(encoding1[field], value1);
    encoding1[field] = value2;

    await sender.setParameters(param1);
    const param2 = sender.getParameters();
    validateSenderRtpParameters(param2);
    const encoding2 = getFirstEncoding(param2);
    assert_equals(encoding2[field], value2);
  }, desc + ' with RTCRtpTransceiverInit');

  promise_test(async t => {
    const pc = new RTCPeerConnection();
    t.add_cleanup(() => pc.close());
    const {
      sender
    } = pc.addTransceiver(kind);
    await doOfferAnswerExchange(t, pc);

    const initParam = sender.getParameters();
    validateSenderRtpParameters(initParam);
    initParam.encodings[0][field] = value1;
    await sender.setParameters(initParam);

    const param1 = sender.getParameters();
    validateSenderRtpParameters(param1);
    const encoding1 = getFirstEncoding(param1);

    assert_equals(encoding1[field], value1);
    encoding1[field] = value2;

    await sender.setParameters(param1);
    const param2 = sender.getParameters();
    validateSenderRtpParameters(param2);
    const encoding2 = getFirstEncoding(param2);
    assert_equals(encoding2[field], value2);
  }, desc + ' without RTCRtpTransceiverInit');
}
