self.addEventListener('fetch', function(event) {
    var res = new Response('<!DOCTYPE html>\n<h1 id=\'testid\'>test</h1>');
    res.headers.delete('content-type');
    event.respondWith(res);
  });
