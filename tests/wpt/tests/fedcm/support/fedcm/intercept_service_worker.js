var num_overridden = 0;

self.addEventListener('fetch', event => {
  const url = event.request.url;
  if (url.indexOf('query_service_worker_intercepts.html') != -1) {
    event.respondWith(new Response(num_overridden));
  } else if (url.indexOf('fedcm/support') != -1) {
    ++num_overridden;
  }
});
