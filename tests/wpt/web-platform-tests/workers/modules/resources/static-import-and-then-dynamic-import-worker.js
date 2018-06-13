import * as module from './export-on-dynamic-import-script.js';
module.ready.then(() => postMessage(module.importedModules));
