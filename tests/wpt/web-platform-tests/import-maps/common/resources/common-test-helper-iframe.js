// Handle errors around fetching, parsing and registering import maps.
const onScriptError = event => {
  window.registrationResult = {type: 'FetchError', error: event.error};
  return false;
};
window.windowErrorHandler = event => {
  window.registrationResult = {type: 'ParseError', error: event.error};
  return false;
};
window.addEventListener('error', window.windowErrorHandler);

// Handle specifier resolution requests from the parent frame.
// For failures, we post error names and messages instead of error
// objects themselves and re-create error objects later, to avoid
// issues around serializing error objects which is a quite new feature.
window.addEventListener('message', event => {
  if (event.data.action === 'prepareResolve') {
    // To get the result of #resolve-a-module-specifier given a script
    // (with base URL = |baseURL|) and |specifier|, the service worker
    // first serves an importer script with response URL = |baseURL|:
    //     window.importHelper = (specifier) => import(specifier);
    // This is to use |baseURL| as the referringScript's base URL.

    // Step 1. Signal the service worker to serve
    // the importer script for the next fetch request.
    parent.worker.postMessage('serveImporterScript');
  } else if (event.data.action === 'resolve') {
    if (event.data.expectedURL === null ||
        new URL(event.data.expectedURL).protocol === 'https:') {
      // Testing without internal methods:
      // If the resolution is expected to fail (null case here),
      // we can test the failure just by catching the exception.
      // If the expected URL is HTTPS, we can test the result by
      // intercepting requests by service workers.

      // Step 3. Evaluate the importer script as a classic script,
      // in order to prevent |baseURL| from being mapped by import maps.
      const script = document.createElement('script');
      script.onload = () => {
        // Step 4. Trigger dynamic import from |baseURL|.
        importHelper(event.data.specifier)
          .then(module => {
              // Step 5. Service worker responds with a JSON containing
              // the request URL for the dynamic import
              // (= the result of #resolve-a-module-specifier).
              parent.postMessage({type: 'ResolutionSuccess',
                                  result: module.response.url},
                                 '*');
            })
          .catch(e => {
              parent.postMessage(
                  {type: 'Failure', result: e.name, message: e.message},
                  '*');
            });
      };
      script.src = event.data.baseURL;
      document.body.appendChild(script);
    } else {
      // Testing with internal methods.
      // For example, the resolution results are data: URLs.
      if (!event.data.useInternalMethods) {
        parent.postMessage(
            {type: 'Failure',
             result: 'Error',
             message: 'internals.resolveModuleSpecifier is not available'},
            '*');
        return;
      }
      try {
        const result = internals.resolveModuleSpecifier(
          event.data.specifier,
          event.data.baseURL,
          document);
        parent.postMessage(
            {type: 'ResolutionSuccess', result: result}, '*');
      } catch (e) {
        parent.postMessage(
            {type: 'Failure', result: e.name, message: e.message}, '*');
      }
    }
  } else if (event.data.action === 'getParsedImportMap') {
    if (!event.data.useInternalMethods) {
      parent.postMessage(
          {type: 'Failure',
           result: 'Error',
           message: 'internals.getParsedImportMap is not available'},
          '*');
    }
    try {
      parent.postMessage({
          type: 'GetParsedImportMapSuccess',
          result: internals.getParsedImportMap(document)}, '*');
    } catch (e) {
      parent.postMessage(
          {type: 'Failure', result: e.name, message: e.message}, '*');
    }
  } else {
    parent.postMessage({
        type: 'Failure',
        result: 'Error',
        message: 'Invalid Action: ' + event.data.action}, '*');
  }
});
