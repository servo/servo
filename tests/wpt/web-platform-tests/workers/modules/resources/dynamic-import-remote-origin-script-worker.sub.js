// Import a remote origin script.
import('https://{{domains[www1]}}:{{ports[https][0]}}/workers/modules/resources/export-on-load-script.js')
  .then(module => postMessage(module.importedModules))
  .catch(e => postMessage(['ERROR']));
