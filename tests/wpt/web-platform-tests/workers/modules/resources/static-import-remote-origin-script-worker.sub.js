// Import a remote origin script.
import * as module from 'https://{{domains[www1]}}:{{ports[https][0]}}/workers/modules/resources/export-on-load-script.js';
if ('DedicatedWorkerGlobalScope' in self &&
    self instanceof DedicatedWorkerGlobalScope) {
  postMessage(module.importedModules);
} else if (
    'SharedWorkerGlobalScope' in self &&
    self instanceof SharedWorkerGlobalScope) {
  onconnect = e => {
    e.ports[0].postMessage(module.importedModules);
  };
}
