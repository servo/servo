import('./static-import-worker.js')
  .then(module => postMessage('LOADED'));
