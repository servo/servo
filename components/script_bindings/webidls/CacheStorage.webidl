// https://w3c.github.io/ServiceWorker/#cachestorage
[Pref="dom_serviceworker_enabled", SecureContext, Exposed=(Window,Worker)]
interface CacheStorage {
  [NewObject] Promise<boolean> has(DOMString cacheName);
};