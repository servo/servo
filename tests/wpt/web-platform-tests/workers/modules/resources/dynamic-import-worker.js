import('./export-on-load-script.js')
  .then(module => postMessage(module.importedModules));
