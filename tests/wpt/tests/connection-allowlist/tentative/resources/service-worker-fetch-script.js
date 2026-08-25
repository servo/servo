self.addEventListener('fetch', (e) => {
  if (e.request.url.includes('blank-with-cors.html')) {
    e.respondWith(fetch(e.request));
  }
});

self.addEventListener('message', async (e) => {
  const url = e.data;
  try {
    const r = await fetch(url, { mode: 'cors', credentials: 'omit' });
    e.source.postMessage({ url: url, success: r.ok });
  } catch (err) {
    e.source.postMessage({ url: url, success: false, error: err.name });
  }
});
