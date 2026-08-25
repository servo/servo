'use strict';

// Dedicated worker
if (typeof postMessage === 'function') {
  onmessage = event => {
    switch(event.data.type) {
      case 'ready':
        navigator.serial.getPorts().then(
            () => postMessage({ type: 'availability-result', enabled: true }),
            error => postMessage ({ type: 'availability-result', enabled: false }));
        break;
    }
  };
}
