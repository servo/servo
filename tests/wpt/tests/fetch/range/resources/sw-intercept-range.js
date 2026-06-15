addEventListener('fetch', event => {
  const url = new URL(event.request.url);
  if (url.searchParams.get('action') !== 'echo-range') {
    return;
  }

  const rangeHeader = event.request.headers.get('Range');
  event.respondWith(new Response(rangeHeader === null ? 'no-range' : rangeHeader, {
    headers: { 'Content-Type': 'text/plain' },
  }));
});
