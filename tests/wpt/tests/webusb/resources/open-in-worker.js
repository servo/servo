importScripts('/resources/test-only-api.js');
importScripts('/webusb/resources/usb-helpers.js');
'use strict';

onmessage = messageEvent => {
  if (messageEvent.data.type === 'ConnectEvent') {
    navigator.usb.addEventListener('connect', connectEvent => {
      connectEvent.device.open().then(() => {
        postMessage({ type: 'Success' });
      }).catch(error => {
        postMessage({ type: `FAIL: open rejected ${error}` });
      });
    });
    postMessage({ type: 'Ready' });
  }
};
