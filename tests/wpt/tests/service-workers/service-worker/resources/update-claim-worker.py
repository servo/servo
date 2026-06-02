import time

script = u'''
// Time stamp: %s
// (This ensures the source text is *not* a byte-for-byte match with any
// previously-fetched version of this script.)

// This no-op fetch handler is necessary to bypass explicitly the no fetch
// handler optimization by which this service worker script can be skipped.
addEventListener('fetch', event => {
    return;
  });

addEventListener('install', event => {
    event.waitUntil(self.skipWaiting());
  });

addEventListener('activate', event => {
    event.waitUntil(self.clients.claim());
  });'''


def main(request, response):
  return [(b'Content-Type', b'application/javascript')], script % time.time()
