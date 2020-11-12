let serveImporterScript = false;

self.addEventListener('message', event => {
  serveImporterScript = true;
  event.source.postMessage('Done');
});

self.addEventListener('fetch', event => {
    if (event.request.url.indexOf('test-helper-iframe.js') >= 0) {
      return;
    }
    if (serveImporterScript) {
      serveImporterScript = false;
      event.respondWith(
        new Response(
          'window.importHelper = (specifier) => import(specifier);',
          {headers: {'Content-Type': 'text/javascript'}}
        ));
    } else {
      event.respondWith(
        new Response(
          'export const response = ' +
              JSON.stringify({url: event.request.url}) + ';',
          {headers: {'Access-Control-Allow-Origin': '*',
                     'Content-Type': 'text/javascript'}}
        ));
    }
});
