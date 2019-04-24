'use strict';

// Dedicated worker
if (typeof postMessage === 'function') {
  onmessage = event => {
    switch(event.data.type) {
      case 'ready':
        new IdleDetector().start().then(() => {
          postMessage({ enabled: true });
        }, error => {
          postMessage ({ enabled: false });
        });
        break;
    }
  };
}
