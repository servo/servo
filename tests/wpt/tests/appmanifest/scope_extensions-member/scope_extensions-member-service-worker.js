// Some user agents only offer app installation if there is a SW and it handles
// offline requests.

const cacheVersion = '1.0';
const CACHE_NAME = `cache-v${cacheVersion}`;

// The resources cached by this service worker.
// Only resources on the primary origin are cached. The target pages are loaded
// on scope-extension origins and cannot be served by this service worker.
const resources = [
  'scope_extensions-member-service-worker.js',
  'scope_extensions-member-manual.sub.tentative.html',
  'resources/icon.png',
];

// Load all resources for this service worker.
const precache = async () => {
  const cache = await caches.open(CACHE_NAME);
  await cache.addAll(resources);
};

// Get a resource from the cache.
const fromCache = async request => {
  const cache = await caches.open(CACHE_NAME);
  return await cache.match(request.url);
};

// Attempt to get resources from the network first, fallback to the cache if
// we're offline.
const networkFallbackToCache = async request => {
  try {
    const response = await fetch(request);
    if (response.ok)
      return response;
  } catch {
    // A cached response may still make the installed app usable offline.
  }
  return await fromCache(request) || Response.error();
};

// When we have a new service worker, update the caches and swap immediately.
self.addEventListener('install', e => {
  e.waitUntil(precache().then(() => self.skipWaiting()));
});

// Claim existing clients.
self.addEventListener('activate', e => {
  e.waitUntil(self.clients.claim());
});

// When a resource needs to be fetched, check whether it is contained in the
// cache and return the cached version, otherwise get it from the network.
self.addEventListener('fetch', e => {
  e.respondWith(networkFallbackToCache(e.request));
});
