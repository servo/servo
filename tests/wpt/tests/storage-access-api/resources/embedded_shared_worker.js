"use strict";

// Replies to commands sent over each connected port. Used by the tests that
// check whether a SharedWorker's requests inherit the creating frame's storage
// access.
self.onconnect = (connectEvent) => {
  const port = connectEvent.ports[0];
  port.onmessage = async (message) => {
    switch (message.data.command) {
    case "fetch": {
      const data = await fetch(message.data.url, {mode: 'cors', credentials: 'include'})
        .then((resp) => resp.text());
      port.postMessage({data});
      break;
    }
    case "websocket": {
      const data = await new Promise((resolve, reject) => {
        const ws = new WebSocket(message.data.origin + '/echo-cookie');
        ws.onmessage = (event) => resolve(event.data);
        ws.onerror = () => reject(new Error('Unexpected error event'));
        ws.onclose = (event) => reject('Unexpected close event: ' + JSON.stringify(event));
      }).catch((error) => "error: " + error);
      port.postMessage({data});
      break;
    }
    default:
    }
  };
};
