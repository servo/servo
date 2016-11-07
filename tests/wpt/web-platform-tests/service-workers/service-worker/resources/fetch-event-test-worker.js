function handleString(event) {
  event.respondWith(new Response('Test string'));
}

function handleBlob(event) {
  event.respondWith(new Response(new Blob(['Test blob'])));
}

function handleReferrer(event) {
  event.respondWith(new Response(new Blob(
    ['Referrer: ' + event.request.referrer])));
}

function handleReferrerPolicy(event) {
  event.respondWith(new Response(new Blob(
    ['ReferrerPolicy: ' + event.request.referrerPolicy])));
}

function handleReferrerFull(event) {
  event.respondWith(new Response(new Blob(
    ['Referrer: ' + event.request.referrer + '\n' +
     'ReferrerPolicy: ' + event.request.referrerPolicy])));
}

function handleClientId(event) {
  var body;
  if (event.clientId !== null) {
    body = 'Client ID Found: ' + event.clientId;
  } else {
    body = 'Client ID Not Found';
  }
  event.respondWith(new Response(body));
}

function handleNullBody(event) {
  event.respondWith(new Response());
}

function handleFetch(event) {
  event.respondWith(fetch('other.html'));
}

function handleFormPost(event) {
  event.respondWith(new Promise(function(resolve) {
      event.request.text()
        .then(function(result) {
            resolve(new Response(event.request.method + ':' +
                                 event.request.headers.get('Content-Type') + ':' +
                                 result));
          });
    }));
}

function handleMultipleRespondWith(event) {
  var logForMultipleRespondWith = '';
  for (var i = 0; i < 3; ++i) {
    logForMultipleRespondWith += '(' + i + ')';
    try {
      event.respondWith(new Promise(function(resolve) {
        setTimeout(function() {
          resolve(new Response(logForMultipleRespondWith));
        }, 0);
      }));
    } catch (e) {
      logForMultipleRespondWith += '[' + e.name + ']';
    }
  }
}

var lastResponseForUsedCheck = undefined;

function handleUsedCheck(event) {
  if (!lastResponseForUsedCheck) {
    event.respondWith(fetch('other.html').then(function(response) {
        lastResponseForUsedCheck = response;
        return response;
      }));
  } else {
    event.respondWith(new Response(
        'bodyUsed: ' + lastResponseForUsedCheck.bodyUsed));
  }
}

function handleFragmentCheck(event) {
  var body;
  if (event.request.url.indexOf('#') === -1) {
    body = 'Fragment Not Found';
  } else {
    body = 'Fragment Found';
  }
  event.respondWith(new Response(body));
}

function handleCache(event) {
  event.respondWith(new Response(event.request.cache));
}

function handleEventSource(event) {
  if (event.request.mode === 'navigate') {
    return;
  }
  var data = {
    mode: event.request.mode,
    cache: event.request.cache,
    credentials: event.request.credentials
  };
  var body = 'data:' + JSON.stringify(data) + '\n\n';
  event.respondWith(new Response(body, {
      headers: { 'Content-Type': 'text/event-stream' }
    }
  ));
}

function handleIntegrity(event) {
  event.respondWith(new Response(event.request.integrity));
}

self.addEventListener('fetch', function(event) {
    var url = event.request.url;
    var handlers = [
      { pattern: '?string', fn: handleString },
      { pattern: '?blob', fn: handleBlob },
      { pattern: '?referrerFull', fn: handleReferrerFull },
      { pattern: '?referrerPolicy', fn: handleReferrerPolicy },
      { pattern: '?referrer', fn: handleReferrer },
      { pattern: '?clientId', fn: handleClientId },
      { pattern: '?ignore', fn: function() {} },
      { pattern: '?null', fn: handleNullBody },
      { pattern: '?fetch', fn: handleFetch },
      { pattern: '?form-post', fn: handleFormPost },
      { pattern: '?multiple-respond-with', fn: handleMultipleRespondWith },
      { pattern: '?used-check', fn: handleUsedCheck },
      { pattern: '?fragment-check', fn: handleFragmentCheck },
      { pattern: '?cache', fn: handleCache },
      { pattern: '?eventsource', fn: handleEventSource },
      { pattern: '?integrity', fn: handleIntegrity },
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
      event.respondWith(new Response(new Blob(
        ['Service Worker got an unexpected request: ' + url])));
    }
  });
