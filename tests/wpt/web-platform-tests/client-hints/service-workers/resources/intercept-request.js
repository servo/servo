self.addEventListener('fetch', (event) => {
  result="FAIL";
  if(event.request.headers.has("device-memory"))
    result="PASS";
  event.respondWith(new Response(result));
});
