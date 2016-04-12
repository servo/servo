var activatePromiseResolve;

addEventListener('activate', function(evt) {
  evt.waitUntil(new Promise(function(resolve) {
    activatePromiseResolve = resolve;
  }));
});

addEventListener('message', function(evt) {
  if (typeof activatePromiseResolve === 'function') {
    activatePromiseResolve();
  }
});

addEventListener('fetch', function(evt) {
  evt.respondWith(new Response('Hello world'));
});
