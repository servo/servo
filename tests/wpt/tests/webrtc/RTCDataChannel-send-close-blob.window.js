// META: title=RTCDataChannel.prototype.send with large Blob
// META: script=RTCPeerConnection-helper.js
// META: script=RTCDataChannel-send-close-helper.js
// META: timeout=long

'use strict';

const largeBlob = new Blob([' '.repeat(largeSendDataLength)]);
rtc_data_channel_send_close_test(/*sendData=*/ largeBlob,
                                 /*dataChannelOptions=*/ {});
