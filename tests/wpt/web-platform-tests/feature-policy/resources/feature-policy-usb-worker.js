'use strict';

// Dedicated worker
if (typeof postMessage === 'function') {
  onmessage = event => {
    switch(event.data.type) {
      case 'ready':
        navigator.usb.getDevices().then(
            () => postMessage({ enabled: true }),
            error => postMessage ({ enabled: false }));
        break;
    }
  };
}