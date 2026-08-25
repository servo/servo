// META: title=RTCDataChannel.prototype.send with large ArrayBuffer
// META: script=RTCPeerConnection-helper.js
// META: script=RTCDataChannel-send-close-helper.js
// META: timeout=long

'use strict';

const largeString = ' '.repeat(largeSendDataLength);
const largeArrayBuffer = new TextEncoder('utf-8').encode(largeString);
rtc_data_channel_send_close_test(
    /*sendData=*/ largeArrayBuffer,
    /*dataChannelOptions=*/ {negotiated: true, id: 0});
