// META: script=RTCPeerConnection-helper.js

const largeSendDataLength = 64 * 1024;

function rtc_data_channel_send_close_test(sendData, dataChannelOptions) {
  const mode =
      `${dataChannelOptions.negotiated ? 'Negotiated d' : 'D'}atachannel`;

  // Determine the type of the data being sent.
  let sendDataType = typeof (sendData);
  if (sendDataType === 'object') {
    if (ArrayBuffer.isView(sendData)) {
      sendDataType = 'arraybuffer';
    } else if (sendData instanceof Blob) {
      sendDataType = 'blob';
    }
  }

  // Determine the length of the data being sent.
  let sendDataLength = 0;
  switch (sendDataType) {
    case 'string':
      sendDataLength = sendData.length;
      break;
    case 'arraybuffer':
      sendDataLength = sendData.byteLength;
      break;
    case 'blob':
      sendDataLength = sendData.size;
      break;
  }

  promise_test(
      async t => {
        assert_greater_than(
            sendDataLength, 0,
            '`sendData` must be a string, Blob or ArrayBuffer view.');

        let [channel1, channel2] =
            await createDataChannelPair(t, dataChannelOptions);
        let receivedSize = 0, sentSize = 0;

        channel2.binaryType = 'arraybuffer';
        channel2.onmessage = e => {
          if (typeof e.data === 'string')
            receivedSize += e.data.length;
          else
            receivedSize += e.data.byteLength;
        };

        channel2.onerror = event => {
          assert_unreached(
              `channel2 must not dispatch error events: ${event.error}.`);
        };

        let closePromiseResolve;
        let closePromise = new Promise((resolve, reject) => {
          closePromiseResolve = resolve;
        });
        channel2.onclose = e => {
          closePromiseResolve();
        };

        try {
          while (sentSize < 20 * 1024 * 1024) {
            channel1.send(sendData);
            sentSize += sendDataLength;
          }
        } catch (error) {
          assert_true(error instanceof DOMException);
          assert_equals(error.name, 'OperationError');
        }
        channel1.onerror = event => {
          assert_unreached(
              `channel1 must not dispatch error events: ${event.error}.`);
        };
        channel1.close();

        await closePromise;
        assert_equals(
            receivedSize, sentSize,
            'All the pending sent messages are received after calling close()');
      },
      `${mode} should be able to send and receive all ${
          sendDataType} messages on close`);
}
