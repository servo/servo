// This script is meant to be imported by a module worker. It receives a
// message from the worker and responds with the list of imported modules.
import * as module from './export-on-dynamic-import-script.js';

const sourcePromise = new Promise(resolve => {
  self.onmessage = e => {
    // DedicatedWorkerGlobalScope doesn't fill in e.source,
    // so use e.target instead.
    const source = e.source ? e.source : e.target;
    resolve(source);
  };
});

export let importedModules = ['export-on-dynamic-import-script.js'];
const importedModulesPromise = module.ready
  .then(importedModules => importedModules)
  .catch(error => `Failed to do dynamic import: ${error}`);

Promise.all([sourcePromise, importedModulesPromise]).then(results => {
  const [source, importedModules] = results;
  source.postMessage(importedModules);
});
