'use strict';

// Test is based on the following editor draft:
// https://w3c.github.io/webrtc-pc/archives/20170605/webrtc.html
// https://w3c.github.io/webrtc-stats/archives/20170614/webrtc-stats.html


// This file depends on dictionary-helper.js which should
// be loaded from the main HTML file.

// To improve readability, the WebIDL definitions of the Stats
// dictionaries are modified to annotate with required fields when
// they are required by section 8.6 of webrtc-pc. ID fields are
// also annotated with the stats type that they are linked to.

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
  assert_number_field(stats, 'timeStamp');
  assert_string_field(stats, 'type');
  assert_string_field(stats, 'id');
}

/*
  [webrtc-stats]
  7.1.  RTCRTPStreamStats dictionary
    dictionary RTCRTPStreamStats : RTCStats {
      required  unsigned long      ssrc;
      required  DOMString          mediaType;

      [RTCMediaStreamTrackStats]
      required  DOMString          trackId;

      [RTCTransportStats]
      required  DOMString          transportId;

      [RTCCodecStats]
      required  DOMString          codecId;

                unsigned long      firCount;
                unsigned long      pliCount;
      required  unsigned long      nackCount;
                unsigned long      sliCount;
                unsigned long long qpSum;
    };

  [webrtc-pc]
  8.6.  Mandatory To Implement Stats
    - RTCRTPStreamStats, with attributes ssrc, associateStatsId, isRemote, mediaType,
      mediaTrackId, transportId, codecId, nackCount
 */
function validateRtpStreamStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  assert_unsigned_int_field(stats, 'ssrc');
  assert_string_field(stats, 'mediaType');

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
      required  unsigned long payloadType;
      required  RTCCodecType  codecType;

      [RTCTransportStats]
      DOMString     transportId;

      DOMString     mimeType;
      required  unsigned long clockRate;
      required  unsigned long channels;
      DOMString     sdpFmtpLine;
      DOMString     implementation;
    };

    enum RTCCodecType {
      "encode",
      "decode",
    };

  [webrtc-pc]
  8.6.  Mandatory To Implement Stats
    - RTCCodecStats, with attributes payloadType, codec, clockRate, channels, parameters
 */

function validateCodecStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  assert_unsigned_int_field(stats, 'payloadType');
  assert_enum_field(stats, 'codecType', ['encode', 'decode']);

  validateOptionalIdField(statsReport, stats, 'transportId', 'transport');

  assert_optional_string_field(stats, 'mimeType');
  assert_unsigned_int_field(stats, 'clockRate');
  assert_unsigned_int_field(stats, 'channels');

  assert_optional_string_field(stats, 'sdpFmtpLine');
  assert_optional_string_field(stats, 'implementation');
}

/*
  [webrtc-stats]
  7.3.  RTCReceivedRTPStreamStats dictionary
    dictionary RTCReceivedRTPStreamStats : RTCRTPStreamStats {
      unsigned long      packetsReceived;
      unsigned long long bytesReceived;
      unsigned long      packetsLost;
      double             jitter;
      double             fractionLost;
      unsigned long      packetsDiscarded;
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
 */
function validateReceivedRtpStreamStats(statsReport, stats) {
  validateRtpStreamStats(statsReport, stats);

  assert_optional_unsigned_int_field(stats, 'packetsReceived');
  assert_optional_unsigned_int_field(stats, 'bytesReceived');
  assert_optional_unsigned_int_field(stats, 'packetsLost');

  assert_optional_number_field(stats, 'jitter');
  assert_optional_number_field(stats, 'fractionLost');

  assert_optional_unsigned_int_field(stats, 'packetsDiscarded');
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
      required  unsigned long      packetsReceived;
      required  unsigned long long bytesReceived;
      required  unsigned long      packetsLost;
      required  double             jitter;
      required  unsigned long      packetsDiscarded;

      [RTCRemoteOutboundRTPStreamStats]
      DOMString           remoteId;

      unsigned long       framesDecoded;
      DOMHighResTimeStamp lastPacketReceivedTimestamp;
    };

  [webrtc-pc]
  8.6.  Mandatory To Implement Stats
    - RTCInboundRTPStreamStats, with all required attributes from RTCRTPStreamStats,
      and also attributes packetsReceived, bytesReceived, packetsLost, jitter,
      packetsDiscarded
 */
function validateInboundRtpStreamStats(statsReport, stats) {
  validateReceivedRtpStreamStats(statsReport, stats);

  assert_unsigned_int_field(stats, 'packetsReceived');
  assert_unsigned_int_field(stats, 'bytesReceived');
  assert_unsigned_int_field(stats, 'packetsLost');
  assert_number_field(stats, 'jitter');
  assert_unsigned_int_field(stats, 'packetsDiscarded');

  validateOptionalIdField(statsReport, stats, 'remoteId', 'remote-outbound-rtp');

  assert_optional_unsigned_int_field(stats, 'framesDecoded');
  assert_optional_number_field(stats, 'lastPacketReceivedTimeStamp');
}

/*
  [webrtc-stats]
  7.5.  RTCRemoteInboundRTPStreamStats dictionary
    dictionary RTCRemoteInboundRTPStreamStats : RTCReceivedRTPStreamStats {
      [RTCOutboundRTPStreamStats]
      DOMString localId;

      double    roundTripTime;
    };
 */

function validateRemoteInboundRtpStreamStats(statsReport, stats) {
  validateReceivedRtpStreamStats(statsReport, stats);

  validateOptionalIdField(statsReport, stats, 'localId', 'outbound-rtp');
  assert_optional_number_field(stats, 'roundTripTime');
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
 */
function validateSentRtpStreamStats(statsReport, stats) {
  validateRtpStreamStats(statsReport, stats);

  assert_optional_unsigned_int_field(stats, 'packetsSent');
  assert_optional_unsigned_int_field(stats, 'packetsDiscardedOnSend');
  assert_optional_unsigned_int_field(stats, 'bytesSent');
  assert_optional_unsigned_int_field(stats, 'bytesDiscardedOnSend');
}

/*
  [webrtc-stats]
  7.7.  RTCOutboundRTPStreamStats dictionary
    dictionary RTCOutboundRTPStreamStats : RTCSentRTPStreamStats {
      required  unsigned long      packetsSent;
      required  unsigned long long bytesSent;

      [RTCRemoteInboundRTPStreamStats]
      DOMString           remoteId;

      DOMHighResTimeStamp lastPacketSentTimestamp;
      double              targetBitrate;
      unsigned long       framesEncoded;
      double              totalEncodeTime;
      double              averageRTCPInterval;
    };

  [webrtc-pc]
  8.6.  Mandatory To Implement Stats
    - RTCOutboundRTPStreamStats, with all required attributes from RTCRTPStreamStats,
      and also attributes packetsSent, bytesSent, roundTripTime
 */
function validateOutboundRtpStreamStats(statsReport, stats) {
  validateOptionalIdField(statsReport, stats, 'remoteId', 'remote-inbound-rtp');

  assert_unsigned_int_field(stats, 'packetsSent');
  assert_unsigned_int_field(stats, 'bytesSent');

  assert_optional_number_field(stats, 'lastPacketSentTimestamp');
  assert_optional_number_field(stats, 'targetBitrate');
  assert_optional_unsigned_int_field(stats, 'framesEncoded');
  assert_optional_number_field(stats, 'totalEncodeTime');
  assert_optional_number_field(stats, 'averageRTCPInterval');
}

/*
  [webrtc-stats]
  7.8.  RTCRemoteOutboundRTPStreamStats dictionary
    dictionary RTCRemoteOutboundRTPStreamStats : RTCSentRTPStreamStats {
      [RTCInboundRTPStreamStats]
      DOMString           localId;

      DOMHighResTimeStamp remoteTimestamp;
    };
 */
function validateRemoteOutboundRtpStreamStats(statsReport, stats) {
  validateSentRtpStreamStats(statsReport, stats);

  validateOptionalIdField(statsReport, stats, 'localId', 'inbound-rtp');
  assert_optional_number_field(stats, 'remoteTimeStamp');
}

/*
  [webrtc-stats]
  7.9.  RTCRTPContributingSourceStats
    dictionary RTCRTPContributingSourceStats : RTCStats {
      unsigned long contributorSsrc;

      [RTCInboundRTPStreamStats]
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
      required  unsigned long dataChannelsOpened;
      required  unsigned long dataChannelsClosed;
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
      required  DOMString           streamIdentifier;

      [RTCMediaStreamTrackStats]
      required  sequence<DOMString> trackIds;
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
      required  DOMString           trackIdentifier;
      required  boolean             remoteSource;
      required  boolean             ended;
      required  boolean             detached;
                DOMString           kind;
                DOMHighResTimeStamp estimatedPlayoutTimestamp;
      required  unsigned long       frameWidth;
      required  unsigned long       frameHeight;
      required  double              framesPerSecond;
                unsigned long       framesCaptured;
      required  unsigned long       framesSent;
      required  unsigned long       framesReceived;
      required  unsigned long       framesDecoded;
      required  unsigned long       framesDropped;
      required  unsigned long       framesCorrupted;
                unsigned long       partialFramesLost;
                unsigned long       fullFramesLost;
      required  double              audioLevel;
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
    - RTCMediaStreamTrackStats, with attributes trackIdentifier, remoteSource, ended,
      detached, ssrcIds, frameWidth, frameHeight, framesPerSecond, framesSent,
      framesReceived, framesDecoded, framesDropped, framesCorrupted, audioLevel
 */

function validateMediaStreamTrackStats(stats, stat) {
  validateRtcStats(statsReport, stats);

  assert_string_field(stat, 'trackIdentifier');
  assert_boolean_field(stat, 'remoteSource');
  assert_boolean_field(stat, 'ended');
  assert_boolean_field(stat, 'detached');

  assert_optional_string_field(stat, 'kind');
  assert_optional_number_field(stat, 'estimatedPlayoutTimestamp');

  assert_unsigned_int_field(stat, 'frameWidth');
  assert_unsigned_int_field(stat, 'frameHeight');
  assert_number_field(stat, 'framesPerSecond');

  assert_optional_unsigned_int_field(stat, 'framesCaptured');
  assert_unsigned_int_field(stat, 'frameSent');
  assert_unsigned_int_field(stat, 'frameReceived');
  assert_unsigned_int_field(stat, 'frameDecoded');
  assert_unsigned_int_field(stat, 'frameDropped');
  assert_unsigned_int_field(stat, 'frameCorrupted');

  assert_optional_unsigned_int_field(stat, 'partialFramesLost');
  assert_optional_unsigned_int_field(stat, 'fullFramesLost');

  assert_number_field(stat, 'audioLevel');
  assert_optional_number_field(stat, 'totalAudioEnergy');
  assert_optional_boolean_field(stat, 'voiceActivityFlag');
  assert_optional_number_field(stat, 'echoReturnLoss');
  assert_optional_number_field(stat, 'echoReturnLossEnhancement');

  assert_optional_unsigned_int_field(stat, 'totalSamplesSent');
  assert_optional_unsigned_int_field(stat, 'totalSamplesReceived');
  assert_optional_number_field(stat, 'totalSamplesDuration');
  assert_optional_unsigned_int_field(stat, 'concealedSamples');
  assert_optional_unsigned_int_field(stat, 'concealmentEvents');
  assert_optional_number_field(stat, 'jitterBufferDelay');

  assert_optional_enum_field(stats, 'priority',
    ['very-low', 'low', 'medium', 'high']);
}

/*
  [webrtc-stats]
  7.13. RTCDataChannelStats dictionary
    dictionary RTCDataChannelStats : RTCStats {
      required  DOMString           label;
      required  DOMString           protocol;
      required  long                datachannelid;

      [RTCTransportStats]
                DOMString           transportId;

      required  RTCDataChannelState state;
      required  unsigned long       messagesSent;
      required  unsigned long long  bytesSent;
      required  unsigned long       messagesReceived;
      required  unsigned long long  bytesReceived;
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
  assert_int_field(stats, 'datachannelid');

  validateOptionalIdField(statsReport, stats, 'transportId', 'transport');

  assert_enum_field(stats, 'state',
    ['connecting', 'open', 'closing', 'closed']);

  assert_unsigned_int_field(stats, 'messageSent');

  assert_unsigned_int_field(stats, 'messageSent');
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
      required  unsigned long long    bytesSent;
      required  unsigned long long    bytesReceived;

      [RTCTransportStats]
      required  DOMString             rtcpTransportStatsId;

                RTCIceRole            iceRole;
                RTCDtlsTransportState dtlsState;

      [RTCIceCandidatePairStats]
      required  DOMString             selectedCandidatePairId;

      [RTCCertificateStats]
      required  DOMString             localCertificateId;

      [RTCCertificateStats]
      required  DOMString             remoteCertificateId;
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
    - RTCTransportStats, with attributes bytesSent, bytesReceived, rtcpTransportStatsId,
      activeConnection, selectedCandidatePairId, localCertificateId, remoteCertificateId
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
  validateIdField(stateReport, stats, 'localCertificateId', 'certificate');
  validateIdField(stateReport, stats, 'remoteCertificateId', 'certificate');
}

/*
  [webrtc-stats]
  7.15. RTCIceCandidateStats dictionary
    dictionary RTCIceCandidateStats : RTCStats {
      [RTCTransportStats]
                DOMString           transportId;

                boolean             isRemote;
      required  DOMString           ip;
      required  long                port;
      required  DOMString           protocol;
      required  RTCIceCandidateType candidateType;
      required  long                priority;
      required  DOMString           url;
                DOMString           relayProtocol;
                boolean             deleted = false;
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
    - RTCIceCandidateStats, with attributes ip, port, protocol, candidateType, priority,
      url
 */

function validateIceCandidateStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  validateOptionalIdField(statsReport, stats, 'transportId', 'transport');
  assert_optional_boolean_field(stats, 'isRemote');

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
      [RTCTransportStats]
      required  DOMString                     transportId;

      [RTCIceCandidateStats]
      required  DOMString                     localCandidateId;

      [RTCIceCandidateStats]
      required  DOMString                     remoteCandidateId;

      required  RTCStatsIceCandidatePairState state;
      required  unsigned long long            priority;
      required  boolean                       nominated;
                unsigned long                 packetsSent;
                unsigned long                 packetsReceived;
      required  unsigned long long            bytesSent;
      required  unsigned long long            bytesReceived;
                DOMHighResTimeStamp           lastPacketSentTimestamp;
                DOMHighResTimeStamp           lastPacketReceivedTimestamp;
                DOMHighResTimeStamp           firstRequestTimestamp;
                DOMHighResTimeStamp           lastRequestTimestamp;
                DOMHighResTimeStamp           lastResponseTimestamp;
      required  double                        totalRoundTripTime;
      required  double                        currentRoundTripTime;
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
      remoteCandidateId, state, priority, nominated, writable, readable, bytesSent,
      bytesReceived, totalRtt, currentRtt
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
      required  DOMString fingerprint;
      required  DOMString fingerprintAlgorithm;
      required  DOMString base64Certificate;
      required  DOMString issuerCertificateId;
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
