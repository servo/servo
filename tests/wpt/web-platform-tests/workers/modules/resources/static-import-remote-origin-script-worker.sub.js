// Import a remote origin script.
import * as module from 'https://{{domains[www1]}}:{{ports[https][0]}}/workers/modules/resources/export-on-load-script.js';
postMessage(module.importedModules);
