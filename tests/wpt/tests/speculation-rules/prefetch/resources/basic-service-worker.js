const swOption = new URL(location.href).searchParams.get('sw');

if (swOption !== 'no-fetch-handler') {
  self.addEventListener('fetch', event => {

    // Intercept the prefetched and navigated URLs only (e.g. do not intercept
    // subresource requests related to `/common/dispatcher/dispatcher.js`).
    if (!event.request.url.includes('counting-executor.py')) {
      return;
    }

    if (swOption === 'fetch-handler') {
      event.respondWith(fetch(event.request));
    } else if (swOption === 'fetch-handler-modify-url') {
      // The "Sec-Purpose: prefetch" header is dropped in fetch-handler-modify-*
      // cases in Step 33 of // https://fetch.spec.whatwg.org/#dom-request
      // because it's a https://fetch.spec.whatwg.org/#forbidden-request-header
      const url = new URL(event.request.url);
      url.searchParams.set('intercepted', 'true');
      event.respondWith(fetch(url, {headers: event.request.headers}));
    } else if (swOption === 'fetch-handler-modify-referrer') {
      event.respondWith(fetch(event.request,
          {referrer: new URL('/intercepted', location.href).href}));
    } else {
      // Do nothing to fallback to the network.
    }
  });
}
