// This script is meant to be imported by a module worker. It receives a
// message from the worker and responds with the list of imported modules.
const code =
  "const sourcePromise = new Promise(resolve => {" +
  "  self.onmessage = e => {" +
  "    const source = e.source ? e.source : e.target;" +
  "    resolve(source);" +
  "  };" +
  "});" +
  "const importedModulesPromise =" +
  "  import('./export-on-load-script.js')" +
  "    .then(module => module.importedModules)" +
  "    .catch(error => `Failed to do dynamic import: ${error}`);" +
  "Promise.all([sourcePromise, importedModulesPromise]).then(results => {" +
  "  const [source, importedModules] = results;" +
  "  source.postMessage(importedModules);" +
  "});";
eval(code);
