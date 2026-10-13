// To be dynamically imported. This script is the initiator of the resources
// loaded here.
import './module-script-imported.js?label=dynamic-module-importer-static';
import('./module-script-imported.js?label=dynamic-module-importer-dynamic');

// Dynamically add a module script element that loads the target.
const script = document.createElement('script');
script.type = 'module';
script.src =
    '/resource-timing/resources/module-script-imported.js?label=dynamic-module-importer-add-script';
document.head.appendChild(script);
