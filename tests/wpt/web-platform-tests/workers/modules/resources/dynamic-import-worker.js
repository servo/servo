import('./post-message-on-load-worker.js')
  .then(module => postMessage('LOADED'));
