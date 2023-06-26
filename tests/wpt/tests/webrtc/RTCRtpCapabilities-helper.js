'use strict'

// Test is based on the following editor draft:
// https://w3c.github.io/webrtc-pc/archives/20170605/webrtc.html

// This file depends on dictionary-helper.js which should
// be loaded from the main HTML file.

/*
  5.2.  RTCRtpSender Interface
    dictionary RTCRtpCapabilities {
      sequence<RTCRtpCodecCapability>           codecs;
      sequence<RTCRtpHeaderExtensionCapability> headerExtensions;
    };

    dictionary RTCRtpCodecCapability {
      DOMString      mimeType;
      unsigned long  clockRate;
      unsigned short channels;
      DOMString      sdpFmtpLine;
    };

    dictionary RTCRtpHeaderExtensionCapability {
      DOMString uri;
    };
 */

function validateRtpCapabilities(capabilities) {
  assert_array_field(capabilities, 'codecs');
  for(const codec of capabilities.codecs) {
    validateCodecCapability(codec);
  }

  assert_greater_than(capabilities.codecs.length, 0,
    'Expect at least one codec capability available');

  assert_array_field(capabilities, 'headerExtensions');
  for(const headerExt of capabilities.headerExtensions) {
    validateHeaderExtensionCapability(headerExt);
  }
}

function validateCodecCapability(codec) {
  assert_optional_string_field(codec, 'mimeType');
  assert_optional_unsigned_int_field(codec, 'clockRate');
  assert_optional_unsigned_int_field(codec, 'channels');
  assert_optional_string_field(codec, 'sdpFmtpLine');
}

function validateHeaderExtensionCapability(headerExt) {
  assert_optional_string_field(headerExt, 'uri');
}
