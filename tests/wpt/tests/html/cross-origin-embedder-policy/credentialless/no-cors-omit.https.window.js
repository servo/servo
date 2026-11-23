// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js

// Under COEP: credentialless, a cross-origin no-cors response doesn't need
// Cross-Origin-Resource-Policy when its request omitted credentials.
// https://fetch.spec.whatwg.org/#cross-origin-resource-policy-internal-check

const resourceURL = () =>
  get_host_info().HTTPS_REMOTE_ORIGIN + "/common/square.png?" + token();

test(() => {
  assert_true(crossOriginIsolated);
}, "The document is cross-origin isolated");

promise_test(async () => {
  await fetch(resourceURL(), { mode: "no-cors", credentials: "omit" });
}, "fetch() of a cross-origin no-cors resource without credentials or CORP");

promise_test(async test => {
  const cacheName = token();
  const cache = await caches.open(cacheName);
  test.add_cleanup(() => caches.delete(cacheName));
  const request = new Request(resourceURL(), { mode: "no-cors", credentials: "omit" });
  await cache.put(request, await fetch(request));
  assert_not_equals(await cache.match(request.url), undefined);
}, "CacheStorage match() of a cross-origin no-cors resource without credentials or CORP");
