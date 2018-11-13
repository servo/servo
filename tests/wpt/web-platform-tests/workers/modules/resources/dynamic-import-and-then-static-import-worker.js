import('./export-on-static-import-script.js')
  .then(module => postMessage(module.importedModules));
