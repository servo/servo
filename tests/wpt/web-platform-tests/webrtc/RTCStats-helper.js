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
      "certificate",
      "ice-server"
    };
 */
const statsValidatorTable = {
  'codec': validateCodecStats,
  'inbound-rtp': validateInboundRtpStreamStats,
  'outbound-rtp': validateOutboundRtpStreamStats,
  'remote-inbound-rtp': validateRemoteInboundRtpStreamStats,
  'remote-outbound-rtp': validateRemoteOutboundRtpStreamStats,
  'media-source': validateMediaSourceStats,
  'csrc': validateContributingSourceStats,
  'peer-connection': validatePeerConnectionStats,
  'data-channel': validateDataChannelStats,
  'transceiver': validateTransceiverStats,
  'sender': validateSenderStats,
  'receiver': validateReceiverStats,
  'transport': validateTransportStats,
  'candidate-pair': validateIceCandidatePairStats,
  'local-candidate': validateIceCandidateStats,
  'remote-candidate': validateIceCandidateStats,
  'certificate': validateCertificateStats,
  'ice-server': validateIceServerStats
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
  7.1.  RTCRtpStreamStats dictionary
    dictionary RTCRtpStreamStats : RTCStats {
      unsigned long       ssrc;
      DOMString           kind;
      DOMString           transportId;
      DOMString           codecId;
    };

    kind of type DOMString
      Either "audio" or "video".

  [webrtc-pc]
  8.6.  Mandatory To Implement Stats
    - RTCRtpStreamStats, with attributes ssrc, kind, transportId, codecId
 */
function validateRtpStreamStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  assert_unsigned_int_field(stats, 'ssrc');
  assert_string_field(stats, 'kind');
  assert_enum_field(stats, 'kind', ['audio', 'video'])

  validateIdField(statsReport, stats, 'transportId', 'transport');
  validateIdField(statsReport, stats, 'codecId', 'codec');

}

/*
  [webrtc-stats]
  7.2.  RTCCodecStats dictionary
    dictionary RTCCodecStats : RTCStats {
      required unsigned long payloadType;
      RTCCodecType  codecType;
      required DOMString     transportId;
      required DOMString     mimeType;
      unsigned long clockRate;
      unsigned long channels;
      DOMString     sdpFmtpLine;
    };

    enum RTCCodecType {
      "encode",
      "decode",
    };

  [webrtc-pc]
  8.6.  Mandatory To Implement Stats
    - RTCCodecStats, with attributes payloadType, codecType, mimeType, clockRate, channels, sdpFmtpLine
 */

function validateCodecStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  assert_unsigned_int_field(stats, 'payloadType');
  assert_optional_enum_field(stats, 'codecType', ['encode', 'decode']);

  validateOptionalIdField(statsReport, stats, 'transportId', 'transport');

  assert_string_field(stats, 'mimeType');
  assert_unsigned_int_field(stats, 'clockRate');
  assert_unsigned_int_field(stats, 'channels');

  assert_string_field(stats, 'sdpFmtpLine');
}

/*
  [webrtc-stats]
  7.3.  RTCReceivedRtpStreamStats dictionary
    dictionary RTCReceivedRtpStreamStats : RTCRtpStreamStats {
      unsigned long long   packetsReceived;
      long long            packetsLost;
      double               jitter;
      unsigned long long   packetsDiscarded;
      unsigned long long   packetsRepaired;
      unsigned long long   burstPacketsLost;
      unsigned long long   burstPacketsDiscarded;
      unsigned long        burstLossCount;
      unsigned long        burstDiscardCount;
      double               burstLossRate;
      double               burstDiscardRate;
      double               gapLossRate;
      double               gapDiscardRate;
      unsigned long        framesDropped;
      unsigned long        partialFramesLost;
      unsigned long        fullFramesLost;
    };

    [webrtc-pc]
    8.6.  Mandatory To Implement Stats
      - RTCReceivedRtpStreamStats, with all required attributes from its
        inherited dictionaries, and also attributes packetsReceived,
        packetsLost, jitter, packetsDiscarded, framesDropped
 */
function validateReceivedRtpStreamStats(statsReport, stats) {
  validateRtpStreamStats(statsReport, stats);

  assert_unsigned_int_field(stats, 'packetsReceived');
  assert_unsigned_int_field(stats, 'packetsLost');

  assert_number_field(stats, 'jitter');

  assert_unsigned_int_field(stats, 'packetsDiscarded');
  assert_unsigned_int_field(stats, 'framesDropped');

  assert_optional_unsigned_int_field(stats, 'packetsRepaired');
  assert_optional_unsigned_int_field(stats, 'burstPacketsLost');
  assert_optional_unsigned_int_field(stats, 'burstPacketsDiscarded');
  assert_optional_unsigned_int_field(stats, 'burstLossCount');
  assert_optional_unsigned_int_field(stats, 'burstDiscardCount');

  assert_optional_number_field(stats, 'burstLossRate');
  assert_optional_number_field(stats, 'burstDiscardRate');
  assert_optional_number_field(stats, 'gapLossRate');
  assert_optional_number_field(stats, 'gapDiscardRate');

  assert_optional_unsigned_int_field(stats, 'partialFramesLost');
  assert_optional_unsigned_int_field(stats, 'fullFramesLost');
}

/*
  [webrtc-stats]
  7.4.  RTCInboundRtpStreamStats dictionary
    dictionary RTCInboundRtpStreamStats : RTCReceivedRtpStreamStats {
      DOMString            trackId;
      DOMString            receiverId;
      DOMString            remoteId;
      unsigned long        framesDecoded;
      unsigned long        keyFramesDecoded;
      unsigned long        frameWidth;
      unsigned long        frameHeight;
      unsigned long        frameBitDepth;
      double               framesPerSecond;
      unsigned long long   qpSum;
      double               totalDecodeTime;
      double               totalInterFrameDelay;
      double               totalSquaredInterFrameDelay;
      boolean              voiceActivityFlag;
      DOMHighResTimeStamp  lastPacketReceivedTimestamp;
      double               averageRtcpInterval;
      unsigned long long   headerBytesReceived;
      unsigned long long   fecPacketsReceived;
      unsigned long long   fecPacketsDiscarded;
      unsigned long long   bytesReceived;
      unsigned long long   packetsFailedDecryption;
      unsigned long long   packetsDuplicated;
      record<USVString, unsigned long long> perDscpPacketsReceived;
      unsigned long        nackCount;
      unsigned long        firCount;
      unsigned long        pliCount;
      unsigned long        sliCount;
      DOMHighResTimeStamp  estimatedPlayoutTimestamp;
      double        jitterBufferDelay;
      unsigned long long   jitterBufferEmittedCount;
      unsigned long long   totalSamplesReceived;
      unsigned long long   samplesDecodedWithSilk;
      unsigned long long   samplesDecodedWithCelt;
      unsigned long long   concealedSamples;
      unsigned long long   silentConcealedSamples;
      unsigned long long   concealmentEvents;
      unsigned long long   insertedSamplesForDeceleration;
      unsigned long long   removedSamplesForAcceleration;
      double               audioLevel;
      double               totalAudioEnergy;
      double               totalSamplesDuration;
      unsigned long        framesReceived;
      DOMString            decoderImplementation;
    };

  [webrtc-pc]
  8.6.  Mandatory To Implement Stats
    - RTCInboundRtpStreamStats, with all required attributes from its inherited
      dictionaries, and also attributes receiverId, remoteId, framesDecoded, nackCount, framesReceived, bytesReceived, totalAudioEnergy, totalSampleDuration
 */
function validateInboundRtpStreamStats(statsReport, stats) {
  validateReceivedRtpStreamStats(statsReport, stats);
  validateOptionalIdField(statsReport, stats, 'trackId', 'track');
  validateIdField(statsReport, stats, 'receiverId', 'receiver');
  validateIdField(statsReport, stats, 'remoteId', 'remote-outbound-rtp');
  assert_unsigned_int_field(stats, 'framesDecoded');
  assert_optional_unsigned_int_field(stats, 'keyFramesDecoded');
  assert_optional_unsigned_int_field(stats, 'frameWidth');
  assert_optional_unsigned_int_field(stats, 'frameHeight');
  assert_optional_unsigned_int_field(stats, 'frameBitDepth');
  assert_optional_number_field(stats, 'framesPerSecond');
  assert_optional_unsigned_int_field(stats, 'qpSum');
  assert_optional_number_field(stats, 'totalDecodeTime');
  assert_optional_number_field(stats, 'totalInterFrameDelay');
  assert_optional_number_field(stats, 'totalSquaredInterFrameDelay');

  assert_optional_boolean_field(stats, 'voiceActivityFlag');

  assert_optional_number_field(stats, 'lastPacketReceivedTimeStamp');
  assert_optional_number_field(stats, 'averageRtcpInterval');

  assert_optional_unsigned_int_field(stats, 'fecPacketsReceived');
  assert_optional_unsigned_int_field(stats, 'fecPacketsDiscarded');
  assert_unsigned_int_field(stats, 'bytesReceived');
  assert_optional_unsigned_int_field(stats, 'packetsFailedDecryption');
  assert_optional_unsigned_int_field(stats, 'packetsDuplicated');

  assert_optional_dict_field(stats, 'perDscpPacketsReceived');
  if (stats['perDscpPacketsReceived']) {
    Object.keys(stats['perDscpPacketsReceived'])
      .forEach(k =>
               assert_equals(typeof k, 'string', 'Expect keys of perDscpPacketsReceived to be strings')
              );
    Object.values(stats['perDscpPacketsReceived'])
      .forEach(v =>
               assert_true(Number.isInteger(v) && (v >= 0), 'Expect values of perDscpPacketsReceived to be strings')
              );
  }

  assert_unsigned_int_field(stats, 'nackCount');

  assert_optional_unsigned_int_field(stats, 'firCount');
  assert_optional_unsigned_int_field(stats, 'pliCount');
  assert_optional_unsigned_int_field(stats, 'sliCount');

  assert_optional_number_field(stats, 'estimatedPlayoutTimestamp');
  assert_optional_number_field(stats, 'jitterBufferDelay');
  assert_optional_unsigned_int_field(stats, 'jitterBufferEmittedCount');
  assert_optional_unsigned_int_field(stats, 'totalSamplesReceived');
  assert_optional_unsigned_int_field(stats, 'samplesDecodedWithSilk');
  assert_optional_unsigned_int_field(stats, 'samplesDecodedWithCelt');
  assert_optional_unsigned_int_field(stats, 'concealedSamples');
  assert_optional_unsigned_int_field(stats, 'silentConcealedSamples');
  assert_optional_unsigned_int_field(stats, 'concealmentEvents');
  assert_optional_unsigned_int_field(stats, 'insertedSamplesForDeceleration');
  assert_optional_unsigned_int_field(stats, 'removedSamplesForAcceleration');
  assert_optional_number_field(stats, 'audioLevel');
  assert_optional_number_field(stats, 'totalAudioEnergy');
  assert_optional_number_field(stats, 'totalSamplesDuration');
  assert_unsigned_int_field(stats, 'framesReceived');
  assert_optional_string_field(stats, 'decoderImplementation');
}

/*
  [webrtc-stats]
  7.5.  RTCRemoteInboundRtpStreamStats dictionary
    dictionary RTCRemoteInboundRtpStreamStats : RTCReceivedRtpStreamStats {
      DOMString            localId;
      double               roundTripTime;
      double               totalRoundTripTime;
      double               fractionLost;
      unsigned long long   reportsReceived;
      unsigned long long   roundTripTimeMeasurements;
    };

  [webrtc-pc]
  8.6.  Mandatory To Implement Stats
    - RTCRemoteInboundRtpStreamStats, with all required attributes from its
      inherited dictionaries, and also attributes localId, roundTripTime
 */
function validateRemoteInboundRtpStreamStats(statsReport, stats) {
  validateReceivedRtpStreamStats(statsReport, stats);

  validateIdField(statsReport, stats, 'localId', 'outbound-rtp');
  assert_number_field(stats, 'roundTripTime');
  assert_optional_number_field(stats, 'totalRoundTripTime');
  assert_optional_number_field(stats, 'fractionLost');
  assert_optional_unsigned_int_field(stats, 'reportsReceived');
  assert_optional_unsigned_int_field(stats, 'roundTripTimeMeasurements');
}

/*
  [webrtc-stats]
  7.6.  RTCSentRtpStreamStats dictionary
    dictionary RTCSentRtpStreamStats : RTCRtpStreamStats {
      unsigned long      packetsSent;
      unsigned long long bytesSent;
    };

    [webrtc-pc]
    8.6.  Mandatory To Implement Stats
      - RTCSentRtpStreamStats, with all required attributes from its inherited
        dictionaries, and also attributes packetsSent, bytesSent
 */
function validateSentRtpStreamStats(statsReport, stats) {
  validateRtpStreamStats(statsReport, stats);

  assert_unsigned_int_field(stats, 'packetsSent');
  assert_unsigned_int_field(stats, 'bytesSent');
}

/*
  [webrtc-stats]
  7.7.  RTCOutboundRtpStreamStats dictionary
    dictionary RTCOutboundRtpStreamStats : RTCSentRtpStreamStats {
      DOMString            mediaSourceId;
      DOMString            senderId;
      DOMString            remoteId;
      DOMString            rid;
      DOMHighResTimeStamp  lastPacketSentTimestamp;
      unsigned long long   headerBytesSent;
      unsigned long        packetsDiscardedOnSend;
      unsigned long long   bytesDiscardedOnSend;
      unsigned long        fecPacketsSent;
      unsigned long long   retransmittedPacketsSent;
      unsigned long long   retransmittedBytesSent;
      double               targetBitrate;
      unsigned long long   totalEncodedBytesTarget;
      unsigned long        frameWidth;
      unsigned long        frameHeight;
      unsigned long        frameBitDepth;
      double               framesPerSecond;
      unsigned long        framesSent;
      unsigned long        hugeFramesSent;
      unsigned long        framesEncoded;
      unsigned long        keyFramesEncoded;
      unsigned long        framesDiscardedOnSend;
      unsigned long long   qpSum;
      unsigned long long   totalSamplesSent;
      unsigned long long   samplesEncodedWithSilk;
      unsigned long long   samplesEncodedWithCelt;
      boolean              voiceActivityFlag;
      double               totalEncodeTime;
      double               totalPacketSendDelay;
      double               averageRtcpInterval;
      RTCQualityLimitationReason          qualityLimitationReason;
      record<DOMString, double> qualityLimitationDurations;
      unsigned long        qualityLimitationResolutionChanges;
      record<USVString, unsigned long long> perDscpPacketsSent;
      unsigned long        nackCount;
      unsigned long        firCount;
      unsigned long        pliCount;
      unsigned long        sliCount;
      DOMString            encoderImplementation;
    };
    Obsolete members:
    partial dictionary RTCOutboundStreamStats {
      DOMString            trackId;
    };
    [webrtc-pc]
    8.6.  Mandatory To Implement Stats
      - RTCOutboundRtpStreamStats, with all required attributes from its
        inherited dictionaries, and also attributes senderId, remoteId, framesEncoded, nackCount, framesSent
 */
function validateOutboundRtpStreamStats(statsReport, stats) {
  validateSentRtpStreamStats(statsReport, stats)

  validateOptionalIdField(statsReport, stats, 'mediaSourceId', 'media-source');
  validateIdField(statsReport, stats, 'senderId', 'sender');
  validateIdField(statsReport, stats, 'remoteId', 'remote-inbound-rtp');

  assert_optional_string_field(stats, 'rid');

  assert_optional_number_field(stats, 'lastPacketSentTimestamp');
  assert_optional_unsigned_int_field(stats, 'headerBytesSent');
  assert_optional_unsigned_int_field(stats, 'packetsDiscardedOnSend');
  assert_optional_unsigned_int_field(stats, 'bytesDiscardedOnSend');
  assert_optional_unsigned_int_field(stats, 'fecPacketsSent');
  assert_optional_unsigned_int_field(stats, 'retransmittedPacketsSent');
  assert_optional_unsigned_int_field(stats, 'retransmittedBytesSent');
  assert_optional_number_field(stats, 'targetBitrate');
  assert_optional_unsigned_int_field(stats, 'totalEncodedBytesTarget');
  if (stats['kind'] === 'video') {
    assert_optional_unsigned_int_field(stats, 'frameWidth');
    assert_optional_unsigned_int_field(stats, 'frameHeight');
    assert_optional_unsigned_int_field(stats, 'frameBitDepth');
    assert_optional_number_field(stats, 'framesPerSecond');
    assert_unsigned_int_field(stats, 'framesSent');
    assert_optional_unsigned_int_field(stats, 'hugeFramesSent');
    assert_unsigned_int_field(stats, 'framesEncoded');
    assert_optional_unsigned_int_field(stats, 'keyFramesEncoded');
    assert_optional_unsigned_int_field(stats, 'framesDiscardedOnSend');
    assert_optional_unsigned_int_field(stats, 'qpSum');
  } else   if (stats['kind'] === 'audio') {
    assert_optional_unsigned_int_field(stats, 'totalSamplesSent');
    assert_optional_unsigned_int_field(stats, 'samplesEncodedWithSilk');
    assert_optional_unsigned_int_field(stats, 'samplesEncodedWithCelt');
    assert_optional_boolean_field(stats, 'voiceActivityFlag');
  }
  assert_optional_number_field(stats, 'totalEncodeTime');
  assert_optional_number_field(stats, 'totalPacketSendDelay');
  assert_optional_number_field(stats, 'averageRTCPInterval');

  if (stats['kind'] === 'video') {
    assert_optional_enum_field(stats, 'qualityLimitationReason', ['none', 'cpu', 'bandwidth', 'other']);

    assert_optional_dict_field(stats, 'qualityLimitationDurations');
    if (stats['qualityLimitationDurations']) {
      Object.keys(stats['qualityLimitationDurations'])
        .forEach(k =>
                 assert_equals(typeof k, 'string', 'Expect keys of qualityLimitationDurations to be strings')
                );
      Object.values(stats['qualityLimitationDurations'])
        .forEach(v =>
                 assert_equals(typeof num, 'number', 'Expect values of qualityLimitationDurations to be numbers')
                );
    }

    assert_optional_unsigned_int_field(stats, 'qualityLimitationResolutionChanges');
    }
  assert_unsigned_int_field(stats, 'nackCount');
  assert_optional_dict_field(stats, 'perDscpPacketsSent');
  if (stats['perDscpPacketsSent']) {
    Object.keys(stats['perDscpPacketsSent'])
      .forEach(k =>
               assert_equals(typeof k, 'string', 'Expect keys of perDscpPacketsSent to be strings')
              );
    Object.values(stats['perDscpPacketsSent'])
      .forEach(v =>
               assert_true(Number.isInteger(v) && (v >= 0), 'Expect values of perDscpPacketsSent to be strings')
              );
  }

  assert_optional_unsigned_int_field(stats, 'firCount');
  assert_optional_unsigned_int_field(stats, 'pliCount');
  assert_optional_unsigned_int_field(stats, 'sliCount');
  assert_optional_string_field(stats, 'encoderImplementation');
  // Obsolete stats
  validateOptionalIdField(statsReport, stats, 'trackId', 'track');
}

/*
  [webrtc-stats]
  7.8.  RTCRemoteOutboundRtpStreamStats dictionary
    dictionary RTCRemoteOutboundRtpStreamStats : RTCSentRtpStreamStats {
      DOMString           localId;
      DOMHighResTimeStamp remoteTimestamp;
      unsigned long long  reportsSent;
    };

  [webrtc-pc]
  8.6.  Mandatory To Implement Stats
    - RTCRemoteOutboundRtpStreamStats, with all required attributes from its
      inherited dictionaries, and also attributes localId, remoteTimestamp
 */
function validateRemoteOutboundRtpStreamStats(statsReport, stats) {
  validateSentRtpStreamStats(statsReport, stats);

  validateIdField(statsReport, stats, 'localId', 'inbound-rtp');
  assert_number_field(stats, 'remoteTimeStamp');
  assert_optional_unsigned_int_field(stats, 'reportsSent');
}

/*
  [webrtc-stats]
  7.11 RTCMediaSourceStats dictionary
  dictionary RTCMediaSourceStats : RTCStats {
      DOMString       trackIdentifier;
      DOMString       kind;
  };

  dictionary RTCAudioSourceStats : RTCMediaSourceStats {
       double       audioLevel;
       double       totalAudioEnergy;
       double       totalSamplesDuration;
       double       echoReturnLoss;
       double       echoReturnLossEnhancement;
  };

  dictionary RTCVideoSourceStats : RTCMediaSourceStats {
      unsigned long   width;
      unsigned long   height;
      unsigned long   bitDepth;
      unsigned long   frames;
      // see https://github.com/w3c/webrtc-stats/issues/540
      double   framesPerSecond;
  };

  [webrtc-pc]
  8.6.  Mandatory To Implement Stats
  RTCMediaSourceStats with attributes trackIdentifier, kind
  RTCAudioSourceStats, with all required attributes from its inherited dictionaries and totalAudioEnergy, totalSamplesDuration
  RTCVideoSourceStats, with all required attributes from its inherited dictionaries and width, height, framesPerSecond
*/
function validateMediaSourceStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);
  assert_string_field(stats, 'trackIdentifier');
  assert_enum_field(stats, 'kind', ['audio', 'video']);

  if (stats.kind === 'audio') {
    assert_optional_number_field(stats, 'audioLevel');
    assert_number_field(stats, 'totalAudioEnergy');
    assert_number_field(stats, 'totalSamplesDuration');
    assert_optional_number_field(stats, 'echoReturnLoss');
    assert_optional_number_field(stats, 'echoReturnLossEnhancement');
  } else if (stats.kind === 'video') {
    assert_unsigned_int_field(stats, 'width');
    assert_unsigned_int_field(stats, 'height');
    assert_optional_unsigned_int_field(stats, 'bitDpeth');
    assert_optional_unsigned_int_field(stats, 'frames');
    assert_number_field(stats, 'framesPerSecond');
  }
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

/* [webrtc-stats]
  7.16 RTCRtpTransceiverStats dictionary
  dictionary RTCRtpTransceiverStats {
    DOMString senderId;
    DOMString receiverId;
    DOMString mid;
  };
*/
function validateTransceiverStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);
  validateOptionalIdField(statsReport, stats, 'senderId', 'sender');
  validateOptionalIdField(statsReport, stats, 'receiverId', 'sender');
  assert_optional_string_field(stats, 'mid');
}

/*
  [webrtc-stats]
  dictionary RTCMediaHandlerStats : RTCStats {
      DOMString           trackIdentifier;
      boolean      remoteSource;
      boolean      ended;
      DOMString           kind;
      RTCPriorityType     priority;
  };
  dictionary RTCVideoHandlerStats : RTCMediaHandlerStats {
  };
  dictionary RTCAudioHandlerStats : RTCMediaHandlerStats {
  };
  Used from validateSenderStats and validateReceiverStats

  [webrtc-priority]
  enum RTCPriorityType {
    "very-low",
    "low",
    "medium",
    "high"
  };

  [webrtc-pc]
  MTI:
  RTCMediaHandlerStats with attributes trackIdentifier
  RTCAudioHandlerStats, with all required attributes from its inherited dictionaries
  RTCVideoHandlerStats, with all required attributes from its inherited dictionaries

*/
function validateMediaHandlerStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);
  assert_string_field(stats, 'trackIdentifier');
  assert_optional_boolean_field(stats, 'remoteSource');
  assert_optional_boolean_field(stats, 'ended');
  assert_optional_string_field(stats, 'kind');
  assert_enum_field(stats, 'priority', ['very-low', 'low', 'medium', 'high']);
}

/*
 [webrtc-stats]
  dictionary RTCAudioSenderStats : RTCAudioHandlerStats {
      DOMString           mediaSourceId;
  };
  dictionary RTCVideoSenderStats : RTCVideoHandlerStats {
      DOMString           mediaSourceId;
  };

  [webrtc-pc]
  MTI:
  RTCVideoSenderStats, with all required attributes from its inherited dictionaries
*/
function validateSenderStats(statsReport, stats) {
  validateMediaHandlerStats(statsReport, stats);
  validateOptionalIdField(statsReport, stats, 'mediaSourceId', 'media-source');
}

/*
 [webrtc-stats]
  dictionary RTCAudioReceiverStats : RTCAudioHandlerStats {
  };
  dictionary RTCVideoReceiverStats : RTCVideoHandlerStats {
  };

  [webrtc-pc]
  MTI:
  RTCVideoReceiverStats, with all required attributes from its inherited dictionaries
*/
function validateReceiverStats(statsReport, stats) {
  validateMediaHandlerStats(statsReport, stats);
}


/*
  [webrtc-stats]
  7.13. RTCDataChannelStats dictionary
    dictionary RTCDataChannelStats : RTCStats {
      DOMString           label;
      DOMString           protocol;
      // see https://github.com/w3c/webrtc-stats/issues/541
      unsigned short      dataChannelIdentifier;
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
    - RTCDataChannelStats, with attributes label, protocol, datachannelIdentifier, state,
      messagesSent, bytesSent, messagesReceived, bytesReceived
 */
function validateDataChannelStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  assert_string_field(stats, 'label');
  assert_string_field(stats, 'protocol');
  assert_unsigned_int_field(stats, 'dataChannelIdentifier');

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
      unsigned long long    packetsSent;
      unsigned long long    packetsReceived;
      unsigned long long    bytesSent;
      unsigned long long    bytesReceived;
      DOMString             rtcpTransportStatsId;
      RTCIceRole            iceRole;
      RTCDtlsTransportState dtlsState;
      DOMString             selectedCandidatePairId;
      DOMString             localCertificateId;
      DOMString             remoteCertificateId;
      DOMString             tlsVersion;
      DOMString             dtlsCipher;
      DOMString             srtpCipher;
      DOMString             tlsGroup;
      unsigned long         selectedCandidatePairChanges;
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
      "unknown",
      "controlling",
      "controlled"
    };

  8.6.  Mandatory To Implement Stats
    - RTCTransportStats, with attributes bytesSent, bytesReceived,
      selectedCandidatePairId, localCertificateId,
      remoteCertificateId
 */
function validateTransportStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  assert_optional_unsigned_int_field(stats, 'packetsSent');
  assert_optional_unsigned_int_field(stats, 'packetsReceived');
  assert_unsigned_int_field(stats, 'bytesSent');
  assert_unsigned_int_field(stats, 'bytesReceived');

  validateOptionalIdField(statsReport, stats, 'rtcpTransportStatsId',
                          'transport');

  assert_optional_enum_field(stats, 'iceRole',
                             ['unknown', 'controlling', 'controlled']);

  assert_optional_enum_field(stats, 'dtlsState',
    ['new', 'connecting', 'connected', 'closed', 'failed']);

  validateIdField(statsReport, stats, 'selectedCandidatePairId', 'candidate-pair');
  validateIdField(statsReport, stats, 'localCertificateId', 'certificate');
  validateIdField(statsReport, stats, 'remoteCertificateId', 'certificate');
  assert_optional_string_field(stats, 'tlsVersion');
  assert_optional_string_field(stats, 'dtlsCipher');
  assert_optional_string_field(stats, 'srtpCipher');
  assert_optional_string_field(stats, 'tlsGroup');
  assert_optional_unsigned_int_field(stats, 'selectedCandidatePairChanges');
}

/*
  [webrtc-stats]
  7.15. RTCIceCandidateStats dictionary
    dictionary RTCIceCandidateStats : RTCStats {
      required DOMString  transportId;
      DOMString?          address;
      long                port;
      DOMString           protocol;
      RTCIceCandidateType candidateType;
      long                priority;
      DOMString           url;
      DOMString           relayProtocol;
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
    - RTCIceCandidateStats, with attributes address, port, protocol, candidateType, url
 */
function validateIceCandidateStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  validateIdField(statsReport, stats, 'transportId', 'transport');
  // The address is mandatory to implement, but is allowed to be null
  // when hidden for privacy reasons.
  if (stats.address != null) {
    // Departure from strict spec reading:
    // This field is populated in a racy manner in Chrome.
    // We allow it to be present or not present for the time being.
    // TODO(https://bugs.chromium.org/1092721): Become consistent.
    assert_optional_string_field(stats, 'address');
  }
  assert_unsigned_int_field(stats, 'port');
  assert_string_field(stats, 'protocol');

  assert_enum_field(stats, 'candidateType',
    ['host', 'srflx', 'prflx', 'relay']);

  assert_optional_int_field(stats, 'priority');
  // The url field is mandatory for local candidates gathered from
  // a STUN or TURN server, and MUST NOT be present otherwise.
  // TODO(hta): Improve checking.
  assert_optional_string_field(stats, 'url');
  assert_optional_string_field(stats, 'relayProtocol');
}

/*
  [webrtc-stats]
  7.16. RTCIceCandidatePairStats dictionary
    dictionary RTCIceCandidatePairStats : RTCStats {
      DOMString                     transportId;
      DOMString                     localCandidateId;
      DOMString                     remoteCandidateId;
      RTCStatsIceCandidatePairState state;
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
      unsigned long                 packetsDiscardedOnSend;
      unsigned long long            bytesDiscardedOnSend;    };

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
      remoteCandidateId, state, nominated, bytesSent, bytesReceived, totalRoundTripTime, currentRoundTripTime
   // not including priority per https://github.com/w3c/webrtc-pc/issues/2457
 */
function validateIceCandidatePairStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  validateIdField(statsReport, stats, 'transportId', 'transport');
  validateIdField(statsReport, stats, 'localCandidateId', 'local-candidate');
  validateIdField(statsReport, stats, 'remoteCandidateId', 'remote-candidate');

  assert_enum_field(stats, 'state',
    ['frozen', 'waiting', 'in-progress', 'failed', 'succeeded']);

  assert_boolean_field(stats, 'nominated');
  assert_optional_unsigned_int_field(stats, 'packetsSent');
  assert_optional_unsigned_int_field(stats, 'packetsReceived');
  assert_unsigned_int_field(stats, 'bytesSent');
  assert_unsigned_int_field(stats, 'bytesReceived');

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
  assert_optional_unsigned_int_field(stats, 'packetsDiscardedOnSend');
  assert_optional_unsigned_int_field(stats, 'bytesDiscardedOnSend');
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
  assert_optional_string_field(stats, 'issuerCertificateId');
}

/*
  [webrtc-stats]
  7.30. RTCIceServerStats dictionary
  dictionary RTCIceServerStats : RTCStats {
      DOMString url;
      long port;
      DOMString protocol;
      unsigned long totalRequestsSent;
      unsigned long totalResponsesReceived;
      double totalRoundTripTime;
    };
*/
function validateIceServerStats(statsReport, stats) {
  validateRtcStats(statsReport, stats);

  assert_optional_string_field(stats, 'url');
  assert_optional_int_field(stats, 'port');
  assert_optional_string_field(stats, 'protocol');
  assert_optional_unsigned_int_field(stats, 'totalRequestsSent');
  assert_optional_unsigned_int_field(stats, 'totalResponsesReceived');
  assert_optional_number_field(stats, 'totalRoundTripTime');
}
