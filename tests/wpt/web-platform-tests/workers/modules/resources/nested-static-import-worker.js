// This script is meant to be imported by a module worker. It receives a
// message from the worker and responds with the list of imported modules.
import * as module from './export-on-static-import-script.js';
self.onmessage = e => {
  // DedicatedWorkerGlobalScope doesn't fill in e.source,
  // so use e.target instead.
  const source = e.source ? e.source : e.target;
  source.postMessage(module.importedModules);
};
