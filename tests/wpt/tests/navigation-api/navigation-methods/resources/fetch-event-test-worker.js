self.addEventListener('fetch', function(event) {
  const request = event.request;
  const body =
    `method = ${request.method}, ` +
    `isReloadNavigation = ${request.isReloadNavigation}`;
  event.transitionWhile(new Response(body));
});
