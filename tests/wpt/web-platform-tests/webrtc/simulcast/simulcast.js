'use strict';
/* Helper functions to munge SDP and split the sending track into
 * separate tracks on the receiving end. This can be done in a number
 * of ways, the one used here uses the fact that the MID and RID header
 * extensions which are used for packet routing share the same wire
 * format. The receiver interprets the rids from the sender as mids
 * which allows receiving the different spatial resolutions on separate
 * m-lines and tracks.
 */
const extensionsToFilter = [
  'urn:ietf:params:rtp-hdrext:sdes:mid',
  'urn:ietf:params:rtp-hdrext:sdes:rtp-stream-id',
  'urn:ietf:params:rtp-hdrext:sdes:repaired-rtp-stream-id',
];

function swapRidAndMidExtensionsInSimulcastOffer(offer, rids) {
  const sections = SDPUtils.splitSections(offer.sdp);
  const dtls = SDPUtils.getDtlsParameters(sections[1], sections[0]);
  const ice = SDPUtils.getIceParameters(sections[1], sections[0]);
  const rtpParameters = SDPUtils.parseRtpParameters(sections[1]);

  // The gist of this hack is that rid and mid have the same wire format.
  const rid = rtpParameters.headerExtensions.find(ext => ext.uri === 'urn:ietf:params:rtp-hdrext:sdes:rtp-stream-id');
  rtpParameters.headerExtensions = rtpParameters.headerExtensions.filter(ext => {
    return !extensionsToFilter.includes(ext.uri);
  });
  // This tells the other side that the RID packets are actually mids.
  rtpParameters.headerExtensions.push({id: rid.id, uri: 'urn:ietf:params:rtp-hdrext:sdes:mid', direction: 'sendrecv'});

  // Filter rtx as we have no way to (re)interpret rrid.
  // Not doing this makes probing use RTX, it's not understood and ramp-up is slower.
  rtpParameters.codecs = rtpParameters.codecs.filter(c => c.name.toUpperCase() !== 'RTX');

  let sdp = SDPUtils.writeSessionBoilerplate() +
    SDPUtils.writeDtlsParameters(dtls, 'actpass') +
    SDPUtils.writeIceParameters(ice) +
    'a=group:BUNDLE ' + rids.join(' ') + '\r\n';
  const baseRtpDescription = SDPUtils.writeRtpDescription('video', rtpParameters);
  rids.forEach(rid => {
    sdp += baseRtpDescription +
        'a=mid:' + rid + '\r\n' +
        'a=msid:rid-' + rid + ' rid-' + rid + '\r\n';
  });
  return sdp;
}

function swapRidAndMidExtensionsInSimulcastAnswer(answer, localDescription, rids) {
  const sections = SDPUtils.splitSections(answer.sdp);
  const dtls = SDPUtils.getDtlsParameters(sections[1], sections[0]);
  const ice = SDPUtils.getIceParameters(sections[1], sections[0]);
  const rtpParameters = SDPUtils.parseRtpParameters(sections[1]);

  rtpParameters.headerExtensions = rtpParameters.headerExtensions.filter(ext => {
    return !extensionsToFilter.includes(ext.uri);
  });
  const localMid = SDPUtils.getMid(SDPUtils.splitSections(localDescription.sdp)[1]);
  let sdp = SDPUtils.writeSessionBoilerplate() +
    SDPUtils.writeDtlsParameters(dtls, 'active') +
    SDPUtils.writeIceParameters(ice) +
    'a=group:BUNDLE ' + localMid + '\r\n';
  sdp += SDPUtils.writeRtpDescription('video', rtpParameters);
  sdp += 'a=mid:' + localMid + '\r\n';

  rids.forEach(rid => {
    sdp += 'a=rid:' + rid + ' recv\r\n';
  });
  sdp += 'a=simulcast:recv ' + rids.join(';') + '\r\n';

  // Re-add headerextensions we filtered.
  const headerExtensions = SDPUtils.parseRtpParameters(SDPUtils.splitSections(localDescription.sdp)[1]).headerExtensions;
  headerExtensions.forEach(ext => {
    if (extensionsToFilter.includes(ext.uri)) {
      sdp += 'a=extmap:' + ext.id + ' ' + ext.uri + '\r\n';
    }
  });
  return sdp;
}
