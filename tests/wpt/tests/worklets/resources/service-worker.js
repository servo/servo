self.addEventListener('fetch', e => {
  if (e.request.url.indexOf('/non-existent-worklet-script.js') != -1)
    e.respondWith(fetch('empty-worklet-script.js'));
});
