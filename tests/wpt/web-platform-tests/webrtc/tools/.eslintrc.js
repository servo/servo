module.exports = {
  rules: {
    'no-undef': 1,
    'no-unused-vars': 0
  },
  plugins: [
    'html'
  ],
  env: {
    browser: true,
    es6: true
  },
  globals: {
    // testharness globals
    test: true,
    async_test: true,
    promise_test: true,
    IdlArray: true,
    assert_true: true,
    assert_false: true,
    assert_equals: true,
    assert_not_equals: true,
    assert_array_equals: true,
    assert_in_array: true,
    assert_unreached: true,
    assert_idl_attribute: true,
    assert_own_property: true,
    assert_greater_than: true,
    assert_less_than: true,
    assert_greater_than_equal: true,
    assert_less_than_equal: true,
    assert_approx_equals: true,


    // WebRTC globals
    RTCPeerConnection: true,
    RTCRtpSender: true,
    RTCRtpReceiver: true,
    RTCRtpTransceiver: true,
    RTCIceTransport: true,
    RTCDtlsTransport: true,
    RTCSctpTransport: true,
    RTCDataChannel: true,
    RTCCertificate: true,
    RTCDTMFSender: true,
    RTCError: true,
    RTCTrackEvent: true,
    RTCPeerConnectionIceEvent: true,
    RTCDTMFToneChangeEvent: true,
    RTCDataChannelEvent: true,
    RTCRtpContributingSource: true,
    RTCRtpSynchronizationSource: true,

    // dictionary-helper.js
    assert_unsigned_int_field: true,
    assert_int_field: true,
    assert_string_field: true,
    assert_number_field: true,
    assert_boolean_field: true,
    assert_array_field: true,
    assert_dict_field: true,
    assert_enum_field: true,

    assert_optional_unsigned_int_field: true,
    assert_optional_int_field: true,
    assert_optional_string_field: true,
    assert_optional_number_field: true,
    assert_optional_boolean_field: true,
    assert_optional_array_field: true,
    assert_optional_dict_field: true,
    assert_optional_enum_field: true,

    // identity-helper.sub.js
    parseAssertionResult: true,
    getIdpDomains: true,
    assert_rtcerror_rejection: true,
    hostString: true,

    // RTCConfiguration-helper.js
    config_test: true,

    // RTCDTMFSender-helper.js
    createDtmfSender: true,
    test_tone_change_events: true,
    getTransceiver: true,

    // RTCPeerConnection-helper.js
    countLine: true,
    countAudioLine: true,
    countVideoLine: true,
    countApplicationLine: true,
    similarMediaDescriptions: true,
    assert_is_session_description: true,
    isSimilarSessionDescription: true,
    assert_session_desc_equals: true,
    assert_session_desc_not_equals: true,
    generateOffer: true,
    generateAnswer: true,
    test_state_change_event: true,
    test_never_resolve: true,
    exchangeIceCandidates: true,
    exchangeOfferAnswer: true,
    createDataChannelPair: true,
    awaitMessage: true,
    blobToArrayBuffer: true,
    assert_equals_typed_array: true,
    generateMediaStreamTrack: true,
    getTrackFromUserMedia: true,
    getUserMediaTracksAndStreams: true,
    performOffer: true,
    Resolver: true,

    // RTCRtpCapabilities-helper.js
    validateRtpCapabilities: true,
    validateCodecCapability: true,
    validateHeaderExtensionCapability: true,

    // RTCRtpParameters-helper.js
    validateSenderRtpParameters: true,
    validateReceiverRtpParameters: true,
    validateRtpParameters: true,
    validateEncodingParameters: true,
    validateRtcpParameters: true,
    validateHeaderExtensionParameters: true,
    validateCodecParameters: true,

    // RTCStats-helper.js
    validateStatsReport: true,
    assert_stats_report_has_stats: true,
    findStatsFromReport: true,
    getRequiredStats: true,
    getStatsById: true,
    validateIdField: true,
    validateOptionalIdField: true,
    validateRtcStats: true,
    validateRtpStreamStats: true,
    validateCodecStats: true,
    validateReceivedRtpStreamStats: true,
    validateInboundRtpStreamStats: true,
    validateRemoteInboundRtpStreamStats: true,
    validateSentRtpStreamStats: true,
    validateOutboundRtpStreamStats: true,
    validateRemoteOutboundRtpStreamStats: true,
    validateContributingSourceStats: true,
    validatePeerConnectionStats: true,
    validateMediaStreamStats: true,
    validateMediaStreamTrackStats: true,
    validateDataChannelStats: true,
    validateTransportStats: true,
    validateIceCandidateStats: true,
    validateIceCandidatePairStats: true,
    validateCertificateStats: true,
  }
}
