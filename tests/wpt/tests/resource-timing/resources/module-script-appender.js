// Classic (non-module) script. Defines a helper that loads
// module-script-imported.js by adding a module script element to the document.
function appendModuleScript(label) {
  const script = document.createElement('script');
  script.type = 'module';
  script.src = getUrl(
      `/resource-timing/resources/module-script-imported.js?label=${label}`);
  document.head.appendChild(script);
}
