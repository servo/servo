addEventListener('fetch', evt => {
  if (evt.request.url.includes('dummy')) {
    evt.respondWith(new Response('intercepted'));
  }
});
