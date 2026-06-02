// META: title=RTCDataChannel.prototype.send with large string
// META: script=RTCPeerConnection-helper.js
// META: script=RTCDataChannel-send-close-helper.js
// META: timeout=long

'use strict';

const largeString = ' '.repeat(largeSendDataLength);
rtc_data_channel_send_close_test(
    /*sendData=*/ largeString,
    /*dataChannelOptions=*/ {negotiated: true, id: 0});
