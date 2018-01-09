'use strict';

// Test is based on the following editor draft:
// webrtc-pc 20171130
// webrtc-stats 20171122

// This file depends on dictionary-helper.js which should
// be loaded from the main HTML file.

/*
  [webrtc-stats]
  6.1.  RTCStatsType enum
    enum RTCStatsType {
      "codec",
      "inbound-rtp",
      "outbound-rtp",
      "remote-inbound-rtp",
      "remote-outbound-rtp",
      "csrc",
      "peer-connection",
      "data-channel",
      "stream",
      "track",
      "transport",
      "candidate-pair",
      "local-candidate",
      "remote-candidate",
      "certificate"
    };
 */
const statsValidatorTable = {
  'codec': validateCodecStats,
  'inbound-rtp': validateInboundRtpStreamStats,
  'outbound-rtp': validateOutboundRtpStreamStats,
  'remote-inbound-rtp': validateRemoteInboundRtpStreamStats,
  'remote-outbound-rtp': validateRemoteOutboundRtpStreamStats,
  'csrc': validateContributingSourceStats,
  'peer-connection': validatePeerConnectionStats,
  'data-channel': validateDataChannelStats,
  'stream': validateMediaStreamStats,
  'track': validateMediaStreamTrackStats,
  'transport': validateTransportStats,
  'candidate-pair': validateIceCandidatePairStats,
  'local-candidate': validateIceCandidateStats,
  'remote-candidate': validateIceCandidateStats,
  'certificate': validateCertificateStats
};

// Validate that the stats objects in a stats report
// follows the respective definitions.
// Stats objects with unknown type are ignored and
// only basic validation is done.
function validateStatsReport(statsReport) {
  for(const [id, stats] of statsReport.entries()) {
    assert_equals(stats.id, id,
      'expect stats.id to be the same as the key in statsReport');

    const validator = statsValidatorTable[stats.type];
    if(validator) {
      validator(statsReport, stats);
    } else {
      validateRtcStats(statsReport, stats);
    }
  }
}

// Assert that the stats report have stats objects of
// given types
function assert_stats_report_has_stats(statsReport, statsTypes) {
  const hasTypes = new Set([...statsReport.values()]
    .map(stats => stats.type));

  for(const type of statsTypes) {
    assert_true(hasTypes.has(type),
      `Expect statsReport to contain stats object of type ${type}`);
  }
}

function findStatsFromReport(statsReport, predicate, message) {
  for (const stats of statsReport.values()) {
    if (predicate(stats)) {
      return stats;
    }
  }

  assert_unreached(message || 'none of stats in statsReport satisfy given condition')
}

// Get stats object of type that is expected to be
// found in the statsReport
function getRequiredStats(statsReport, type) {
  for(const stats of statsReport.values()) {
    if(stats.type === type) {
      return stats;
    }
  }

  assert_unreached(`required stats of type ${type} is not found in stats report`);
}

// Get stats object by the stats ID.
// This is used to retreive other stats objects
// linked to a stats object
function getStatsById(statsReport, statsId) {
  assert_true(statsReport.has(statsId),
    `Expect stats report to have stats object with id ${statsId}`);

  return statsReport.get(statsId);
}

// Validate an ID field in a stats object by making sure
// that the linked stats object is found in the stats report
// and have the type field value same as expected type
// It doesn't validate the other fields of the linked stats
// as validateStatsReport already does all validations
function validateIdField(statsReport, stats, field, type) {
  assert_string_field(stats, field);
  const linkedStats = getStatsById(statsReport, stats[field]);
  assert_equals(linkedStats.type, type,
    `Expect linked stats object to have type ${type}`);
}

function validateOptionalIdField(statsReport, stats, field, type) {
  if(stats[field] !== undefined) {
    validateIdField(statsReport, stats, field, type);
  }
}

/*
  [webrtc-pc]
  8.4.  RTCStats Dictionary
    dictionary RTCStats {
      required  DOMHighResTimeStamp timestamp;
      required  RTCStatsType        type;
      required  DOMString           id;
    };
 */
function validateRtcStats(statsReport, stats) {
  assert_number_field(stats, 'timestamp');
  assert_string_field(stats, 'type');
  assert_string_field(stats, 'id');
}

/*
  [webrtc-stats]
  7.1.  RTCRTPStreamStats dictionary
    dictionary RTCRTPStreamStats : RTCStats {
      unsigned long      ssrc;
      DOMString          mediaType;
      DOMString          trackId;
      DOMString          transportId;
      DOMString          codecId;
      unsigned long      firCount;
      unsigned long      pliCount;
      unsigned long      nackCount;
      unsigned long      sliCount;
      unsigned long long qpSum;
    };

    mediaType of type DOMString
      Either "audio" or "video".

  [webrtc-pc]
  8.6.  Mandatory To Implement Stats
    - RTCRTPStreamStats, with attributes ssrc, mediaType, trackId,
      transportId, codecId, nackCount
 */
function validateRtpStreamStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  assert_unsigned_int_field(stats, 'ssrc');
  assert_string_field(stats, 'mediaType');
  assert_enum_field(stats, 'mediaType', ['audio', 'video'])

  validateIdField(statsReport, stats, 'trackId', 'track');
  validateIdField(statsReport, stats, 'transportId', 'transport');
  validateIdField(statsReport, stats, 'codecId', 'codec');

  assert_optional_unsigned_int_field(stats, 'firCount');
  assert_optional_unsigned_int_field(stats, 'pliCount');
  assert_unsigned_int_field(stats, 'nackCount');
  assert_optional_unsigned_int_field(stats, 'sliCount');
  assert_optional_unsigned_int_field(stats, 'qpSum');
}

/*
  [webrtc-stats]
  7.2.  RTCCodecStats dictionary
    dictionary RTCCodecStats : RTCStats {
      unsigned long payloadType;
      RTCCodecType  codecType;
      DOMString     transportId;
      DOMString     mimeType;
      unsigned long clockRate;
      unsigned long channels;
      DOMString     sdpFmtpLine;
      DOMString     implementation;
    };

    enum RTCCodecType {
      "encode",
      "decode",
    };

  [webrtc-pc]
  8.6.  Mandatory To Implement Stats
    - RTCCodecStats, with attributes payloadType, codec, clockRate, channels, sdpFmtpLine
 */

function validateCodecStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  assert_unsigned_int_field(stats, 'payloadType');
  assert_enum_field(stats, 'codecType', ['encode', 'decode']);

  validateOptionalIdField(statsReport, stats, 'transportId', 'transport');

  assert_optional_string_field(stats, 'mimeType');
  assert_unsigned_int_field(stats, 'clockRate');
  assert_unsigned_int_field(stats, 'channels');

  assert_string_field(stats, 'sdpFmtpLine');
  assert_optional_string_field(stats, 'implementation');
}

/*
  [webrtc-stats]
  7.3.  RTCReceivedRTPStreamStats dictionary
    dictionary RTCReceivedRTPStreamStats : RTCRTPStreamStats {
        unsigned long      packetsReceived;
        unsigned long long bytesReceived;
        long               packetsLost;
        double             jitter;
        double             fractionLost;
        unsigned long      packetsDiscarded;
        unsigned long      packetsFailedDecryption;
        unsigned long      packetsRepaired;
        unsigned long      burstPacketsLost;
        unsigned long      burstPacketsDiscarded;
        unsigned long      burstLossCount;
        unsigned long      burstDiscardCount;
        double             burstLossRate;
        double             burstDiscardRate;
        double             gapLossRate;
        double             gapDiscardRate;
    };

    [webrtc-pc]
    8.6.  Mandatory To Implement Stats
      - RTCReceivedRTPStreamStats, with all required attributes from its
        inherited dictionaries, and also attributes packetsReceived,
        bytesReceived, packetsLost, jitter, packetsDiscarded
 */
function validateReceivedRtpStreamStats(statsReport, stats) {
  validateRtpStreamStats(statsReport, stats);

  assert_unsigned_int_field(stats, 'packetsReceived');
  assert_unsigned_int_field(stats, 'bytesReceived');
  assert_unsigned_int_field(stats, 'packetsLost');

  assert_number_field(stats, 'jitter');
  assert_optional_number_field(stats, 'fractionLost');

  assert_unsigned_int_field(stats, 'packetsDiscarded');
  assert_optional_unsigned_int_field(stats, 'packetsFailedDecryption');
  assert_optional_unsigned_int_field(stats, 'packetsRepaired');
  assert_optional_unsigned_int_field(stats, 'burstPacketsLost');
  assert_optional_unsigned_int_field(stats, 'burstPacketsDiscarded');
  assert_optional_unsigned_int_field(stats, 'burstLossCount');
  assert_optional_unsigned_int_field(stats, 'burstDiscardCount');

  assert_optional_number_field(stats, 'burstLossRate');
  assert_optional_number_field(stats, 'burstDiscardRate');
  assert_optional_number_field(stats, 'gapLossRate');
  assert_optional_number_field(stats, 'gapDiscardRate');
}

/*
  [webrtc-stats]
  7.4.  RTCInboundRTPStreamStats dictionary
    dictionary RTCInboundRTPStreamStats : RTCReceivedRTPStreamStats {
      DOMString           remoteId;
      unsigned long       framesDecoded;
      DOMHighResTimeStamp lastPacketReceivedTimestamp;
    };

  [webrtc-pc]
  8.6.  Mandatory To Implement Stats
    - RTCInboundRTPStreamStats, with all required attributes from its inherited
      dictionaries, and also attributes remoteId, framesDecoded
 */
function validateInboundRtpStreamStats(statsReport, stats) {
  validateReceivedRtpStreamStats(statsReport, stats);

  validateIdField(statsReport, stats, 'remoteId', 'remote-outbound-rtp');
  assert_unsigned_int_field(stats, 'framesDecoded');
  assert_optional_number_field(stats, 'lastPacketReceivedTimeStamp');
}

/*
  [webrtc-stats]
  7.5.  RTCRemoteInboundRTPStreamStats dictionary
    dictionary RTCRemoteInboundRTPStreamStats : RTCReceivedRTPStreamStats {
        DOMString localId;
        double    roundTripTime;
    };

  [webrtc-pc]
  8.6.  Mandatory To Implement Stats
    - RTCRemoteInboundRTPStreamStats, with all required attributes from its
      inherited dictionaries, and also attributes localId, roundTripTime
 */
function validateRemoteInboundRtpStreamStats(statsReport, stats) {
  validateReceivedRtpStreamStats(statsReport, stats);

  validateIdField(statsReport, stats, 'localId', 'outbound-rtp');
  assert_number_field(stats, 'roundTripTime');
}

/*
  [webrtc-stats]
  7.6.  RTCSentRTPStreamStats dictionary
    dictionary RTCSentRTPStreamStats : RTCRTPStreamStats {
      unsigned long      packetsSent;
      unsigned long      packetsDiscardedOnSend;
      unsigned long long bytesSent;
      unsigned long long bytesDiscardedOnSend;
    };

    [webrtc-pc]
    8.6.  Mandatory To Implement Stats
      - RTCSentRTPStreamStats, with all required attributes from its inherited
        dictionaries, and also attributes packetsSent, bytesSent
 */
function validateSentRtpStreamStats(statsReport, stats) {
  validateRtpStreamStats(statsReport, stats);

  assert_unsigned_int_field(stats, 'packetsSent');
  assert_optional_unsigned_int_field(stats, 'packetsDiscardedOnSend');
  assert_unsigned_int_field(stats, 'bytesSent');
  assert_optional_unsigned_int_field(stats, 'bytesDiscardedOnSend');
}

/*
  [webrtc-stats]
  7.7.  RTCOutboundRTPStreamStats dictionary
    dictionary RTCOutboundRTPStreamStats : RTCSentRTPStreamStats {
      DOMString           remoteId;
      DOMHighResTimeStamp lastPacketSentTimestamp;
      double              targetBitrate;
      unsigned long       framesEncoded;
      double              totalEncodeTime;
      double              averageRTCPInterval;
    };

    [webrtc-pc]
    8.6.  Mandatory To Implement Stats
      - RTCOutboundRTPStreamStats, with all required attributes from its
        inherited dictionaries, and also attributes remoteId, framesEncoded
 */
function validateOutboundRtpStreamStats(statsReport, stats) {
  validateSentRtpStreamStats(statsReport, stats)

  validateIdField(statsReport, stats, 'remoteId', 'remote-inbound-rtp');

  assert_optional_number_field(stats, 'lastPacketSentTimestamp');
  assert_optional_number_field(stats, 'targetBitrate');
  assert_unsigned_int_field(stats, 'framesEncoded');
  assert_optional_number_field(stats, 'totalEncodeTime');
  assert_optional_number_field(stats, 'averageRTCPInterval');
}

/*
  [webrtc-stats]
  7.8.  RTCRemoteOutboundRTPStreamStats dictionary
    dictionary RTCRemoteOutboundRTPStreamStats : RTCSentRTPStreamStats {
      DOMString           localId;
      DOMHighResTimeStamp remoteTimestamp;
    };

  [webrtc-pc]
  8.6.  Mandatory To Implement Stats
    - RTCRemoteOutboundRTPStreamStats, with all required attributes from its
      inherited dictionaries, and also attributes localId, remoteTimestamp
 */
function validateRemoteOutboundRtpStreamStats(statsReport, stats) {
  validateSentRtpStreamStats(statsReport, stats);

  validateIdField(statsReport, stats, 'localId', 'inbound-rtp');
  assert_number_field(stats, 'remoteTimeStamp');
}

/*
  [webrtc-stats]
  7.9.  RTCRTPContributingSourceStats
    dictionary RTCRTPContributingSourceStats : RTCStats {
      unsigned long contributorSsrc;
      DOMString     inboundRtpStreamId;
      unsigned long packetsContributedTo;
      double        audioLevel;
    };
 */
function validateContributingSourceStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  assert_optional_unsigned_int_field(stats, 'contributorSsrc');

  validateOptionalIdField(statsReport, stats, 'inboundRtpStreamId', 'inbound-rtp');
  assert_optional_unsigned_int_field(stats, 'packetsContributedTo');
  assert_optional_number_field(stats, 'audioLevel');
}

/*
  [webrtc-stats]
  7.10. RTCPeerConnectionStats dictionary
    dictionary RTCPeerConnectionStats : RTCStats {
      unsigned long dataChannelsOpened;
      unsigned long dataChannelsClosed;
      unsigned long dataChannelsRequested;
      unsigned long dataChannelsAccepted;
    };

  [webrtc-pc]
  8.6.  Mandatory To Implement Stats
    - RTCPeerConnectionStats, with attributes dataChannelsOpened, dataChannelsClosed
 */
function validatePeerConnectionStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  assert_unsigned_int_field(stats, 'dataChannelsOpened');
  assert_unsigned_int_field(stats, 'dataChannelsClosed');
  assert_optional_unsigned_int_field(stats, 'dataChannelsRequested');
  assert_optional_unsigned_int_field(stats, 'dataChannelsAccepted');
}

/*
  [webrtc-stats]
  7.11. RTCMediaStreamStats dictionary
    dictionary RTCMediaStreamStats : RTCStats {
      DOMString           streamIdentifier;
      sequence<DOMString> trackIds;
    };

  [webrtc-pc]
  8.6.  Mandatory To Implement Stats
    - RTCMediaStreamStats, with attributes streamIdentifer, trackIds
 */
function validateMediaStreamStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  assert_string_field(stats, 'streamIdentifier');
  assert_array_field(stats, 'trackIds');

  for(const trackId of stats.trackIds) {
    assert_equals(typeof trackId, 'string',
      'Expect trackId elements to be string');

    assert_true(statsReport.has(trackId),
      `Expect stats report to have stats object with id ${trackId}`);

    const trackStats = statsReport.get(trackId);
    assert_equals(trackStats.type, 'track',
      `Expect track stats object to have type 'track'`);
  }
}

/*
  [webrtc-stats]
  7.12. RTCMediaStreamTrackStats dictionary
    dictionary RTCMediaStreamTrackStats : RTCStats {
      DOMString           trackIdentifier;
      boolean             remoteSource;
      boolean             ended;
      boolean             detached;
      DOMString           kind;
      DOMHighResTimeStamp estimatedPlayoutTimestamp;
      unsigned long       frameWidth;
      unsigned long       frameHeight;
      double              framesPerSecond;
      unsigned long       framesCaptured;
      unsigned long       framesSent;
      unsigned long       keyFramesSent;
      unsigned long       framesReceived;
      unsigned long       keyFramesReceived;
      unsigned long       framesDecoded;
      unsigned long       framesDropped;
      unsigned long       framesCorrupted;
      unsigned long       partialFramesLost;
      unsigned long       fullFramesLost;
      double              audioLevel;
      double              totalAudioEnergy;
      boolean             voiceActivityFlag;
      double              echoReturnLoss;
      double              echoReturnLossEnhancement;
      unsigned long long  totalSamplesSent;
      unsigned long long  totalSamplesReceived;
      double              totalSamplesDuration;
      unsigned long long  concealedSamples;
      unsigned long long  concealmentEvents;
      double              jitterBufferDelay;
      RTCPriorityType     priority;
    };

  [webrtc-pc]
  4.9.1.  RTCPriorityType Enum
    enum RTCPriorityType {
      "very-low",
      "low",
      "medium",
      "high"
    };

  8.6.  Mandatory To Implement Stats
    - RTCMediaStreamTrackStats, with attributes trackIdentifier, remoteSource,
      ended, detached, frameWidth, frameHeight, framesPerSecond, framesSent,
      framesReceived, framesDecoded, framesDropped, framesCorrupted, audioLevel
 */

function validateMediaStreamTrackStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  assert_string_field(stats, 'trackIdentifier');
  assert_boolean_field(stats, 'remoteSource');
  assert_boolean_field(stats, 'ended');
  assert_boolean_field(stats, 'detached');

  assert_optional_enum_field(stats, 'kind', ['audio', 'video']);
  assert_optional_number_field(stats, 'estimatedPlayoutTimestamp');

  assert_unsigned_int_field(stats, 'frameWidth');
  assert_unsigned_int_field(stats, 'frameHeight');
  assert_number_field(stats, 'framesPerSecond');

  assert_optional_unsigned_int_field(stats, 'framesCaptured');
  assert_unsigned_int_field(stats, 'framesSent');
  assert_optional_unsigned_int_field(stats, 'keyFramesSent');
  assert_unsigned_int_field(stats, 'framesReceived');
  assert_optional_unsigned_int_field(stats, 'keyFramesReceived');
  assert_unsigned_int_field(stats, 'framesDecoded');
  assert_unsigned_int_field(stats, 'framesDropped');
  assert_unsigned_int_field(stats, 'framesCorrupted');

  assert_optional_unsigned_int_field(stats, 'partialFramesLost');
  assert_optional_unsigned_int_field(stats, 'fullFramesLost');

  assert_number_field(stats, 'audioLevel');
  assert_optional_number_field(stats, 'totalAudioEnergy');
  assert_optional_boolean_field(stats, 'voiceActivityFlag');
  assert_optional_number_field(stats, 'echoReturnLoss');
  assert_optional_number_field(stats, 'echoReturnLossEnhancement');

  assert_optional_unsigned_int_field(stats, 'totalSamplesSent');
  assert_optional_unsigned_int_field(stats, 'totalSamplesReceived');
  assert_optional_number_field(stats, 'totalSamplesDuration');
  assert_optional_unsigned_int_field(stats, 'concealedSamples');
  assert_optional_unsigned_int_field(stats, 'concealmentEvents');
  assert_optional_number_field(stats, 'jitterBufferDelay');

  assert_optional_enum_field(stats, 'priority',
    ['very-low', 'low', 'medium', 'high']);
}

/*
  [webrtc-stats]
  7.13. RTCDataChannelStats dictionary
    dictionary RTCDataChannelStats : RTCStats {
      DOMString           label;
      DOMString           protocol;
      long                dataChannelIdentifier;
      DOMString           transportId;
      RTCDataChannelState state;
      unsigned long       messagesSent;
      unsigned long long  bytesSent;
      unsigned long       messagesReceived;
      unsigned long long  bytesReceived;
    };

  [webrtc-pc]
  6.2. RTCDataChannel
    enum RTCDataChannelState {
      "connecting",
      "open",
      "closing",
      "closed"
    };

  8.6.  Mandatory To Implement Stats
    - RTCDataChannelStats, with attributes label, protocol, datachannelId, state,
      messagesSent, bytesSent, messagesReceived, bytesReceived
 */
function validateDataChannelStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  assert_string_field(stats, 'label');
  assert_string_field(stats, 'protocol');
  assert_int_field(stats, 'dataChannelIdentifier');

  validateOptionalIdField(statsReport, stats, 'transportId', 'transport');

  assert_enum_field(stats, 'state',
    ['connecting', 'open', 'closing', 'closed']);

  assert_unsigned_int_field(stats, 'messagesSent');
  assert_unsigned_int_field(stats, 'bytesSent');
  assert_unsigned_int_field(stats, 'messagesReceived');
  assert_unsigned_int_field(stats, 'bytesReceived');
}

/*
  [webrtc-stats]
  7.14. RTCTransportStats dictionary
    dictionary RTCTransportStats : RTCStats {
      unsigned long         packetsSent;
      unsigned long         packetsReceived;
      unsigned long long    bytesSent;
      unsigned long long    bytesReceived;
      DOMString             rtcpTransportStatsId;
      RTCIceRole            iceRole;
      RTCDtlsTransportState dtlsState;
      DOMString             selectedCandidatePairId;
      DOMString             localCertificateId;
      DOMString             remoteCertificateId;
    };

  [webrtc-pc]
  5.5.  RTCDtlsTransportState Enum
    enum RTCDtlsTransportState {
      "new",
      "connecting",
      "connected",
      "closed",
      "failed"
    };

  5.6.  RTCIceRole Enum
    enum RTCIceRole {
      "controlling",
      "controlled"
    };

  8.6.  Mandatory To Implement Stats
    - RTCTransportStats, with attributes bytesSent, bytesReceived,
      rtcpTransportStatsId, selectedCandidatePairId, localCertificateId,
      remoteCertificateId
 */
function validateTransportStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  assert_optional_unsigned_int_field(stats, 'packetsSent');
  assert_optional_unsigned_int_field(stats, 'packetsReceived');
  assert_unsigned_int_field(stats, 'bytesSent');
  assert_unsigned_int_field(stats, 'bytesReceived');

  validateIdField(statsReport, stats, 'rtcpTransportStatsId', 'transport');

  assert_optional_enum_field(stats, 'iceRole',
    ['controlling', 'controlled']);

  assert_optional_enum_field(stats, 'dtlsState',
    ['new', 'connecting', 'connected', 'closed', 'failed']);

  validateIdField(statsReport, stats, 'selectedCandidatePairId', 'candidate-pair');
  validateIdField(statsReport, stats, 'localCertificateId', 'certificate');
  validateIdField(statsReport, stats, 'remoteCertificateId', 'certificate');
}

/*
  [webrtc-stats]
  7.15. RTCIceCandidateStats dictionary
    dictionary RTCIceCandidateStats : RTCStats {
      DOMString           transportId;
      boolean             isRemote;
      RTCNetworkType      networkType;
      DOMString           ip;
      long                port;
      DOMString           protocol;
      RTCIceCandidateType candidateType;
      long                priority;
      DOMString           url;
      DOMString           relayProtocol;
      boolean             deleted = false;
    };

    enum RTCNetworkType {
      "bluetooth",
      "cellular",
      "ethernet",
      "wifi",
      "wimax",
      "vpn",
      "unknown"
    };

  [webrtc-pc]
  4.8.1.3.  RTCIceCandidateType Enum
    enum RTCIceCandidateType {
      "host",
      "srflx",
      "prflx",
      "relay"
    };

  8.6.  Mandatory To Implement Stats
    - RTCIceCandidateStats, with attributes ip, port, protocol, candidateType,
      priority, url
 */
function validateIceCandidateStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  validateOptionalIdField(statsReport, stats, 'transportId', 'transport');
  assert_optional_boolean_field(stats, 'isRemote');

  assert_optional_enum_field(stats, 'networkType',
    ['bluetooth', 'cellular', 'ethernet', 'wifi', 'wimax', 'vpn', 'unknown'])

  assert_string_field(stats, 'ip');
  assert_int_field(stats, 'port');
  assert_string_field(stats, 'protocol');

  assert_enum_field(stats, 'candidateType',
    ['host', 'srflx', 'prflx', 'relay']);

  assert_int_field(stats, 'priority');
  assert_string_field(stats, 'url');
  assert_optional_string_field(stats, 'relayProtocol');
  assert_optional_boolean_field(stats, 'deleted');
}

/*
  [webrtc-stats]
  7.16. RTCIceCandidatePairStats dictionary
    dictionary RTCIceCandidatePairStats : RTCStats {
      DOMString                     transportId;
      DOMString                     localCandidateId;
      DOMString                     remoteCandidateId;
      RTCStatsIceCandidatePairState state;
      unsigned long long            priority;
      boolean                       nominated;
      unsigned long                 packetsSent;
      unsigned long                 packetsReceived;
      unsigned long long            bytesSent;
      unsigned long long            bytesReceived;
      DOMHighResTimeStamp           lastPacketSentTimestamp;
      DOMHighResTimeStamp           lastPacketReceivedTimestamp;
      DOMHighResTimeStamp           firstRequestTimestamp;
      DOMHighResTimeStamp           lastRequestTimestamp;
      DOMHighResTimeStamp           lastResponseTimestamp;
      double                        totalRoundTripTime;
      double                        currentRoundTripTime;
      double                        availableOutgoingBitrate;
      double                        availableIncomingBitrate;
      unsigned long                 circuitBreakerTriggerCount;
      unsigned long long            requestsReceived;
      unsigned long long            requestsSent;
      unsigned long long            responsesReceived;
      unsigned long long            responsesSent;
      unsigned long long            retransmissionsReceived;
      unsigned long long            retransmissionsSent;
      unsigned long long            consentRequestsSent;
      DOMHighResTimeStamp           consentExpiredTimestamp;
    };

    enum RTCStatsIceCandidatePairState {
      "frozen",
      "waiting",
      "in-progress",
      "failed",
      "succeeded"
    };

  [webrtc-pc]
  8.6.  Mandatory To Implement Stats
    - RTCIceCandidatePairStats, with attributes transportId, localCandidateId,
      remoteCandidateId, state, priority, nominated, bytesSent, bytesReceived, totalRoundTripTime, currentRoundTripTime
 */
function validateIceCandidatePairStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  validateIdField(statsReport, stats, 'transportId', 'transport');
  validateIdField(statsReport, stats, 'localCandidateId', 'local-candidate');
  validateIdField(statsReport, stats, 'remoteCandidateId', 'remote-candidate');

  assert_enum_field(stats, 'state',
    ['frozen', 'waiting', 'in-progress', 'failed', 'succeeded']);

  assert_unsigned_int_field(stats, 'priority');
  assert_boolean_field(stats, 'nominated');
  assert_optional_unsigned_int_field(stats, 'packetsSent');
  assert_optional_unsigned_int_field(stats, 'packetsReceived');
  assert_unsigned_int_field(stats, 'bytesSent');
  assert_unsigned_int_field(stats, 'byteReceived');

  assert_optional_number_field(stats, 'lastPacketSentTimestamp');
  assert_optional_number_field(stats, 'lastPacketReceivedTimestamp');
  assert_optional_number_field(stats, 'firstRequestTimestamp');
  assert_optional_number_field(stats, 'lastRequestTimestamp');
  assert_optional_number_field(stats, 'lastResponseTimestamp');

  assert_number_field(stats, 'totalRoundTripTime');
  assert_number_field(stats, 'currentRoundTripTime');

  assert_optional_number_field(stats, 'availableOutgoingBitrate');
  assert_optional_number_field(stats, 'availableIncomingBitrate');

  assert_optional_unsigned_int_field(stats, 'circuitBreakerTriggerCount');
  assert_optional_unsigned_int_field(stats, 'requestsReceived');
  assert_optional_unsigned_int_field(stats, 'requestsSent');
  assert_optional_unsigned_int_field(stats, 'responsesReceived');
  assert_optional_unsigned_int_field(stats, 'responsesSent');
  assert_optional_unsigned_int_field(stats, 'retransmissionsReceived');
  assert_optional_unsigned_int_field(stats, 'retransmissionsSent');
  assert_optional_unsigned_int_field(stats, 'consentRequestsSent');
  assert_optional_number_field(stats, 'consentExpiredTimestamp');
}

/*
  [webrtc-stats]
  7.17. RTCCertificateStats dictionary
    dictionary RTCCertificateStats : RTCStats {
      DOMString fingerprint;
      DOMString fingerprintAlgorithm;
      DOMString base64Certificate;
      DOMString issuerCertificateId;
    };

  [webrtc-pc]
  8.6.  Mandatory To Implement Stats
    - RTCCertificateStats, with attributes fingerprint, fingerprintAlgorithm,
      base64Certificate, issuerCertificateId
 */
function validateCertificateStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  assert_string_field(stats, 'fingerprint');
  assert_string_field(stats, 'fingerprintAlgorithm');
  assert_string_field(stats, 'base64Certificate');
  assert_string_field(stats, 'issuerCertificateId');
}
