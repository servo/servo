"use strict";

self.addEventListener("activate", event => {
  // start controlling the already loaded page
  event.waitUntil(self.clients.claim());
});

self.addEventListener("fetch", event => {
  const response = new Response("Service worker response", {
    statusText: "OK from serviceworker",
  });
  event.respondWith(response);
});
