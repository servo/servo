import('./export-on-dynamic-import-script.js')
  .then(async module => {
    await module.ready;
    postMessage(module.importedModules);
  });
