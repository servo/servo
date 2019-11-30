importScripts("/resources/testharness.js");

self.addEventListener('fetch', event => {
  if (event.request.url.indexOf('fallback') >= 0) {
    event.addPerformanceEntry(
      performance.mark("network-fallback mark 1",
        { detail: { foo: 'foo' } }));
    event.addPerformanceEntry(
      performance.mark("network-fallback mark 2",
        { detail: { bar: 'bar' } }));
    event.addPerformanceEntry(performance.measure("network-fallback measure",
      {
        start: "network-fallback mark 1", end: "network-fallback mark 2",
        detail: { baz: 'baz' }
      }));
    return;
  } else if (event.request.url.indexOf('fetch-event') >= 0) {
    event.respondWith((async () => {
      event.addPerformanceEntry(performance.mark("fetch-event mark 1",
        { detail: { foo: 'foo' } }));
      const response = await fetch(event.request);

      event.waitUntil(new Promise((resolve) => {
        // Add performance entries after settling a promise for respondWith().
        step_timeout(() => {
          event.addPerformanceEntry(performance.mark("fetch-event mark 2",
            { detail: { bar: 'bar' } }));
          event.addPerformanceEntry(performance.measure("fetch-event measure",
            {
              start: "fetch-event mark 1", end: "fetch-event mark 2",
              detail: { baz: 'baz' }
            }));
          resolve();
        }, 100);
      }));
      return response;
    })());
  }
});
