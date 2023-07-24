// Import a remote origin script.
const import_url = 'https://{{domains[www1]}}:{{ports[https][0]}}/workers/modules/resources/export-referrer-checker.py';
if ('DedicatedWorkerGlobalScope' in self &&
    self instanceof DedicatedWorkerGlobalScope) {
  import(import_url)
      .then(module => postMessage(module.referrer))
      .catch(error => postMessage(`Import failed: ${error}`));
} else if (
    'SharedWorkerGlobalScope' in self &&
    self instanceof SharedWorkerGlobalScope) {
  onconnect = e => {
    import(import_url)
        .then(module => e.ports[0].postMessage(module.referrer))
        .catch(error => e.ports[0].postMessage(`Import failed: ${error}`));
  };
}
