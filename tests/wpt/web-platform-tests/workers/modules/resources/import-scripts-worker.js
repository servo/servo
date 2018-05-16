try {
  importScripts('empty-worker.js');
  postMessage('LOADED');
} catch (e) {
  postMessage(e.name);
}
