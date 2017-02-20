self.addEventListener('install', function(event) {
    var params = JSON.parse(decodeURIComponent(location.search.substring(1)));
    if (!('scopes' in params)) {
      if ('relscopes' in params) {
        params.scopes = params.relscopes.map(s => registration.scope + s);
      } else {
        params.scopes = [registration.scope];
      }
    }
    if (!('origins' in params))
      params.origins = ['*'];
    event.registerForeignFetch(params);
  });

function handle_basic(event) {
  event.respondWith({response: new Response('Foreign Fetch'), origin: event.origin});
}

function handle_null(event) {
  event.respondWith({response: null });
}

function handle_onmessage(event) {
  event.respondWith({origin: event.origin, response:
    new Response('<script>window.onmessage = e => e.ports[0].postMessage("failed");</script>',
                 {headers: {'Content-Type': 'text/html'}})});
}

function handle_fallback(event) {
  // Do nothing.
}

function handle_fetch(event) {
  event.respondWith(
    fetch(event.request).then(response => ({response, origin: event.origin})));
}

function handle_meta(event) {
  var data = {
    origin: event.origin,
    referrer: event.request.referrer
  };
  event.respondWith({response: new Response(JSON.stringify(data)),
                     origin: event.origin});
}

function handle_script(event) {
  event.respondWith({origin: event.origin, response:
    new Response('self.DidLoad("Foreign Fetch");')});
}

self.addEventListener('foreignfetch', function(event) {
    var url = event.request.url;
    var handlers = [
      { pattern: '?basic', fn: handle_basic },
      { pattern: '?null', fn: handle_null },
      { pattern: '?fallback', fn: handle_fallback },
      { pattern: '?fetch', fn: handle_fetch },
      { pattern: '?onmessage', fn: handle_onmessage },
      { pattern: '?meta', fn: handle_meta },
      { pattern: '?script', fn: handle_script }
    ];

    var handler = null;
    for (var i = 0; i < handlers.length; ++i) {
      if (url.indexOf(handlers[i].pattern) != -1) {
        handler = handlers[i];
        break;
      }
    }

    if (handler) {
      handler.fn(event);
    } else {
      event.respondWith({origin: event.origin,
                         response: new Response('unexpected request')});
    }
  });
