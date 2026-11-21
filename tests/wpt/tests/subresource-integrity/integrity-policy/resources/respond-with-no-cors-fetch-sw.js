// Responds to requests carrying a `respond-with` parameter with a credentialed
// no-cors fetch of that URL, i.e. an opaque response whose URL list is hidden
// from the page.
self.addEventListener('fetch', event => {
  const target = new URL(event.request.url).searchParams.get('respond-with');
  if (target) {
    event.respondWith(fetch(target, {mode: 'no-cors', credentials: 'include'}));
  }
});
