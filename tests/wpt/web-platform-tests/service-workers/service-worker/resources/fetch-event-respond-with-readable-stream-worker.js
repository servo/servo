'use strict';

self.addEventListener('fetch', event => {
    if (!event.request.url.match(/body-stream$/))
      return;

    const stream = new ReadableStream({start: controller => {
        const encoder = new TextEncoder();
        controller.enqueue(encoder.encode('PASS'));
        controller.close();
      }});
    event.respondWith(new Response(stream));
  });
