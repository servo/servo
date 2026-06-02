const originalSwOption = new URL(location.href).searchParams.get('sw');
let swOption = originalSwOption;

if (swOption === 'fetch-handler-navigation-preload') {
  self.addEventListener('activate', event => {
    if (self.registration.navigationPreload) {
      event.waitUntil(self.registration.navigationPreload.enable());
    }
  });
}

if (swOption === 'race-fetch-handler' ||
  swOption === 'race-fetch-handler-to-fallback' ||
  swOption === 'race-fetch-handler-modify-url') {
  swOption = swOption.substring('race-'.length);
  self.addEventListener('install', event => {
    event.addRoutes([{
      condition: { urlPattern: { pathname: '/**/counting-executor.py' } },
      source: 'race-network-and-fetch-handler'
    }]);
  });
}

const interceptedRequests = [];

self.addEventListener('message', event => {
  if (event.data === 'getInterceptedRequests') {
    event.source.postMessage(interceptedRequests);
  }
});

if (swOption !== 'no-fetch-handler') {
  self.addEventListener('fetch', event => {

    // Intercept the prefetched and navigated URLs only (e.g. do not intercept
    // subresource requests related to `/common/dispatcher/dispatcher.js`).
    if (!event.request.url.includes('counting-executor.py')) {
      return;
    }

    const headers = {};
    event.request.headers.forEach((value, key) => {
      headers[key] = value;
    });
    const interceptedRequest = {
      request: {
        url: event.request.url,
        headers: headers,
      },
      clientId: event.clientId,
      resultingClientId: event.resultingClientId
    };
    interceptedRequests.push(interceptedRequest);

    if (swOption === 'fetch-handler') {
      event.respondWith(fetch(event.request));
    } else if (swOption === 'fetch-handler-synthetic') {
      const finalUrl = new URL(event.request.url).searchParams.get('location');
      if (finalUrl) {
        event.respondWith(Response.redirect(finalUrl));
      } else {
        // Fallback to the network.
      }
    } else if (swOption === 'fetch-handler-modify-url') {
      // The "Sec-Purpose: prefetch" header is dropped in fetch-handler-modify-*
      // cases in Step 33 of // https://fetch.spec.whatwg.org/#dom-request
      // because it's a https://fetch.spec.whatwg.org/#forbidden-request-header
      const url = new URL(event.request.url);
      url.searchParams.set('intercepted', 'true');
      if (originalSwOption === 'race-fetch-handler-modify-url') {
        // See the comment in `basic.sub.https.html` for delay value.
        url.searchParams.set('delay', '500');
      }
      event.respondWith(fetch(url, {headers: event.request.headers}));
    } else if (swOption === 'fetch-handler-modify-referrer') {
      event.respondWith(fetch(event.request,
          {referrer: new URL('/intercepted', location.href).href}));
    } else if (swOption === 'fetch-handler-navigation-preload') {
      event.respondWith((async () => {
        try {
          if (event.preloadResponse === 'undefined') {
            interceptedRequest.preloadResponse = 'undefined';
            return fetch(event.request);
          }
          const response = await event.preloadResponse;
          if (response) {
            interceptedRequest.preloadResponse = 'resolved';
            return response;
          } else {
            interceptedRequest.preloadResponse = 'resolved to undefined';
            return fetch(event.request);
          }
        } catch(e) {
          interceptedRequest.preloadResponse = 'rejected';
          throw e;
        }
      })());
    } else {
      // Do nothing to fallback to the network.
    }
  });
}
