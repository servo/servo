'use strict';
importScripts("/resources/testharness.js");

self.addEventListener('fetch', event => {
  const url = new URL(event.request.url);
  if (!url.searchParams.has('stream')) return;

  if (url.searchParams.has('use-fetch-stream')) {
    event.respondWith(async function() {
      const response = await fetch('pass.txt');
      return new Response(response.body);
    }());
    return;
  }

  const delayEnqueue = url.searchParams.has('delay');

  const stream = new ReadableStream({
    start(controller) {
      const encoder = new TextEncoder();

      const populate = () => {
        controller.enqueue(encoder.encode('PASS'));
        controller.close();
      }

      if (delayEnqueue) {
        step_timeout(populate, 16);
      }
      else {
        populate();
      }
    }
  });

  event.respondWith(new Response(stream));
});
