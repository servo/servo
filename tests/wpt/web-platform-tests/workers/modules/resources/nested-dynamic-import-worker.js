import('./dynamic-import-worker.js')
  .then(module => postMessage('LOADED'));
