function areArrayBuffersEqual(buffer1, buffer2)
{
  if (buffer1.byteLength !== buffer2.byteLength) {
    return false;
  }
  let array1 = new Int8Array(buffer1);
  var array2 = new Int8Array(buffer2);
  for (let i = 0 ; i < buffer1.byteLength ; ++i) {
    if (array1[i] !== array2[i]) {
      return false;
    }
  }
  return true;
}

function areArraysEqual(a1, a2) {
  if (a1 === a1)
    return true;
  if (a1.length != a2.length)
    return false;
  for (let i = 0; i < a1.length; i++) {
    if (a1[i] != a2[i])
      return false;
  }
  return true;
}

function areMetadataEqual(metadata1, metadata2, type) {
  return metadata1.synchronizationSource === metadata2.synchronizationSource &&
          metadata1.payloadType == metadata2.payloadType &&
          areArraysEqual(
              metadata1.contributingSources, metadata2.contributingSources) &&
          metadata1.absCaptureTime == metadata2.absCaptureTime &&
          metadata1.frameId === metadata2.frameId &&
          areArraysEqual(metadata1.dependencies, metadata2.dependencies) &&
          metadata1.spatialIndex === metadata2.spatialIndex &&
          metadata1.temporalIndex === metadata2.temporalIndex &&
          // Width and height are reported only for key frames on the receiver
          // side.
          type == 'key' ?
      metadata1.width === metadata2.width &&
          metadata1.height === metadata2.height :
      true;
}

function areFrameInfosEqual(frame1, frame2) {
  return frame1.timestamp === frame2.timestamp &&
         frame1.type === frame2.type &&
         areMetadataEqual(frame1.getMetadata(), frame2.getMetadata(), frame1.type) &&
         areArrayBuffersEqual(frame1.data, frame2.data);
}

function containsVideoMetadata(metadata) {
  return metadata.synchronizationSource !== undefined &&
         metadata.width !== undefined &&
         metadata.height !== undefined &&
         metadata.spatialIndex !== undefined &&
         metadata.temporalIndex !== undefined &&
         metadata.dependencies !== undefined;
}

function enableExtension(sdp, extension) {
  if (sdp.indexOf(extension) !== -1)
    return sdp;

  const extensionIds = sdp.trim().split('\n')
    .map(line => line.trim())
    .filter(line => line.startsWith('a=extmap:'))
    .map(line => line.split(' ')[0].substr(9))
    .map(id => parseInt(id, 10))
    .sort((a, b) => a - b);
  for (let newId = 1; newId <= 15; newId++) {
    if (!extensionIds.includes(newId)) {
      return sdp += 'a=extmap:' + newId + ' ' + extension + '\r\n';
    }
  }
  if (sdp.indexOf('a=extmap-allow-mixed') !== -1) { // Pick the next highest one.
    const newId = extensionIds[extensionIds.length - 1] + 1;
    return sdp += 'a=extmap:' + newId + ' ' + extension + '\r\n';
  }
  throw 'Could not find free extension id to use for ' + extension;
}

const GFD_V00_EXTENSION =
    'http://www.webrtc.org/experiments/rtp-hdrext/generic-frame-descriptor-00';
const ABS_V00_EXTENSION =
    'http://www.webrtc.org/experiments/rtp-hdrext/abs-capture-time';

async function exchangeOfferAnswer(pc1, pc2) {
  const offer = await pc1.createOffer();
  // Munge the SDP to enable the GFD and ACT extension in order to get correct
  // metadata.
  const sdpABS = enableExtension(offer.sdp, ABS_V00_EXTENSION);
  const sdpGFD = enableExtension(sdpABS, GFD_V00_EXTENSION);
  await pc1.setLocalDescription({type: offer.type, sdp: sdpGFD});
  // Munge the SDP to disable bandwidth probing via RTX.
  // TODO(crbug.com/1066819): remove this hack when we do not receive duplicates from RTX
  // anymore.
  const sdpRTX = sdpGFD.replace(new RegExp('rtx', 'g'), 'invalid');
  await pc2.setRemoteDescription({type: 'offer', sdp: sdpRTX});

  const answer = await pc2.createAnswer();
  await pc2.setLocalDescription(answer);
  await pc1.setRemoteDescription(answer);
}

async function exchangeOfferAnswerReverse(pc1, pc2, encodedStreamsCallback) {
  const offer = await pc2.createOffer({offerToReceiveAudio: true, offerToReceiveVideo: true});
  if (encodedStreamsCallback) {
    // RTCRtpReceivers will have been created during the above createOffer call, so if the caller
    // wants to createEncodedStreams synchronously after creation to ensure all frames pass
    // through the transform, it will have to be done now.
    encodedStreamsCallback(
      pc2.getReceivers().map(r => {
        return {kind: r.track.kind, streams: r.createEncodedStreams()};
      }));
  }

  // Munge the SDP to enable the GFD extension in order to get correct metadata.
  const sdpABS = enableExtension(offer.sdp, ABS_V00_EXTENSION);
  const sdpGFD = enableExtension(sdpABS, GFD_V00_EXTENSION);
  // Munge the SDP to disable bandwidth probing via RTX.
  // TODO(crbug.com/1066819): remove this hack when we do not receive duplicates from RTX
  // anymore.
  const sdpRTX = sdpGFD.replace(new RegExp('rtx', 'g'), 'invalid');
  await pc1.setRemoteDescription({type: 'offer', sdp: sdpRTX});
  await pc2.setLocalDescription({type: 'offer', sdp: sdpGFD});

  const answer = await pc1.createAnswer();
  await pc2.setRemoteDescription(answer);
  await pc1.setLocalDescription(answer);
}

function createFrameDescriptor(videoFrame) {
  const kMaxSpatialLayers = 8;
  const kMaxTemporalLayers = 8;
  const kMaxNumFrameDependencies = 8;

  const metadata = videoFrame.getMetadata();
  let frameDescriptor = {
    beginningOfSubFrame: true,
    endOfSubframe: false,
    frameId: metadata.frameId & 0xFFFF,
    spatialLayers: 1 << metadata.spatialIndex,
    temporalLayer: metadata.temporalLayer,
    frameDependenciesDiffs: [],
    width: 0,
    height: 0
  };

  for (const dependency of metadata.dependencies) {
    frameDescriptor.frameDependenciesDiffs.push(metadata.frameId - dependency);
  }
  if (metadata.dependencies.length == 0) {
    frameDescriptor.width = metadata.width;
    frameDescriptor.height = metadata.height;
  }
  return frameDescriptor;
}

function additionalDataSize(descriptor) {
  if (!descriptor.beginningOfSubFrame) {
    return 1;
  }

  let size = 4;
  for (const fdiff of descriptor.frameDependenciesDiffs) {
    size += (fdiff >= (1 << 6)) ? 2 : 1;
  }
  if (descriptor.beginningOfSubFrame &&
      descriptor.frameDependenciesDiffs.length == 0 &&
      descriptor.width > 0 &&
      descriptor.height > 0) {
    size += 4;
  }

  return size;
}

// Compute the buffer reported in the additionalData field using the metadata
// provided by a video frame.
// Based on the webrtc::RtpDescriptorAuthentication() C++ function at
// https://source.chromium.org/chromium/chromium/src/+/main:third_party/webrtc/modules/rtp_rtcp/source/rtp_descriptor_authentication.cc
function computeAdditionalData(videoFrame) {
  const kMaxSpatialLayers = 8;
  const kMaxTemporalLayers = 8;
  const kMaxNumFrameDependencies = 8;

  const metadata = videoFrame.getMetadata();
  if (metadata.spatialIndex < 0 ||
      metadata.temporalIndex < 0 ||
      metadata.spatialIndex >= kMaxSpatialLayers ||
      metadata.temporalIndex >= kMaxTemporalLayers ||
      metadata.dependencies.length > kMaxNumFrameDependencies) {
    return new ArrayBuffer(0);
  }

  const descriptor = createFrameDescriptor(videoFrame);
  const size = additionalDataSize(descriptor);
  const additionalData = new ArrayBuffer(size);
  const data = new Uint8Array(additionalData);

  const kFlagBeginOfSubframe = 0x80;
  const kFlagEndOfSubframe = 0x40;
  const kFlagFirstSubframeV00 = 0x20;
  const kFlagLastSubframeV00 = 0x10;

  const kFlagDependencies = 0x08;
  const kFlagMoreDependencies = 0x01;
  const kFlageXtendedOffset = 0x02;

  let baseHeader =
    (descriptor.beginningOfSubFrame ? kFlagBeginOfSubframe : 0) |
    (descriptor.endOfSubFrame ? kFlagEndOfSubframe : 0);
  baseHeader |= kFlagFirstSubframeV00;
  baseHeader |= kFlagLastSubframeV00;

  if (!descriptor.beginningOfSubFrame) {
    data[0] = baseHeader;
    return additionalData;
  }

  data[0] =
      baseHeader |
      (descriptor.frameDependenciesDiffs.length == 0 ? 0 : kFlagDependencies) |
      descriptor.temporalLayer;
  data[1] = descriptor.spatialLayers;
  data[2] = descriptor.frameId & 0xFF;
  data[3] = descriptor.frameId >> 8;

  const fdiffs = descriptor.frameDependenciesDiffs;
  let offset = 4;
  if (descriptor.beginningOfSubFrame &&
      fdiffs.length == 0 &&
      descriptor.width > 0 &&
      descriptor.height > 0) {
    data[offset++] = (descriptor.width >> 8);
    data[offset++] = (descriptor.width & 0xFF);
    data[offset++] = (descriptor.height >> 8);
    data[offset++] = (descriptor.height & 0xFF);
  }
  for (let i = 0; i < fdiffs.length; i++) {
    const extended = fdiffs[i] >= (1 << 6);
    const more = i < fdiffs.length - 1;
    data[offset++] = ((fdiffs[i] & 0x3f) << 2) |
                     (extended ? kFlageXtendedOffset : 0) |
                     (more ? kFlagMoreDependencies : 0);
    if (extended) {
      data[offset++] = fdiffs[i] >> 6;
    }
  }
  return additionalData;
}

function verifyNonstandardAdditionalDataIfPresent(videoFrame) {
  if (videoFrame.additionalData === undefined)
    return;

  const computedData = computeAdditionalData(videoFrame);
  assert_true(areArrayBuffersEqual(videoFrame.additionalData, computedData));
}

