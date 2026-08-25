// To be statically imported. This script is the initiator of the resources
// loaded here.
import './module-script-imported.js?label=static-module-importer-static';
import('./module-script-imported.js?label=static-module-importer-dynamic');

// Dynamically add a module script element that loads the target.
const script = document.createElement('script');
script.type = 'module';
script.src =
    '/resource-timing/resources/module-script-imported.js?label=static-module-importer-add-script';
document.head.appendChild(script);
