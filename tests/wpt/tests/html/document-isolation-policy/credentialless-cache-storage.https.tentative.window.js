// META: timeout=long
// META: variant=?document
// META: variant=?dedicated_worker
// META: variant=?shared_worker
// META: variant=?service_worker
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./resources/common.js

// Fetch a resource and store it into CacheStorage from |storer| context. Then
// check if it can be retrieved via CacheStorage.match from |retriever| context.
const cacheStorageTest = (
  description,
  storer,
  retriever,
  resource_headers,
  request_credential_mode,
  expectation
) => {
  promise_test_parallel(async test => {
    const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;
    const url = cross_origin + "/common/square.png?pipe=" + resource_headers +
      `&${token()}`;
    const this_token = token();

    // Fetch a request from |stored|. Store the opaque response into
    // CacheStorage.
    send(storer, `
      const cache = await caches.open("v1");
      const fetch_request = new Request("${url}", {
        mode: 'no-cors',
        credentials: '${request_credential_mode}'
      });
      const fetch_response = await fetch(fetch_request);
      await cache.put(fetch_request, fetch_response);
      send("${this_token}", "stored");
    `);
    assert_equals(await receive(this_token), "stored");

    // Retrieved it from |retriever|.
    send(retriever, `
      const cache = await caches.open("v1");
      try {
        const response = await cache.match("${url}");
        send("${this_token}", "retrieved");
      } catch (error) {
        send("${this_token}", "error");
      }
    `);
    assert_equals(await receive(this_token), expectation);
  }, description);
};

// Execute the same set of tests for every type of execution contexts:
// Documents, DedicatedWorkers, SharedWorkers, and ServiceWorkers. The results
// should be independent of the context.
const environment = location.search.substr(1);
const constructor = environments[environment];

const context_none = constructor(coep_none)[0];
const context_credentialless = constructor(dip_credentialless)[0];
const context_require_corp = constructor(dip_require_corp)[0];

cacheStorageTest(`[${environment}] none => none`,
  context_none,
  context_none,
  "",
  "include",
  "retrieved");
cacheStorageTest(`[${environment}] none => isolate-and-credentialless`,
  context_none,
  context_credentialless,
  "",
  "include",
  "error");
cacheStorageTest(`[${environment}] none => isolate-and-credentialless (omit)`,
  context_none,
  context_credentialless,
  "",
  "omit",
  "retrieved");
cacheStorageTest(`[${environment}] none => isolate-and-credentialless + CORP`,
  context_none,
  context_credentialless,
  corp_cross_origin,
  "include",
  "retrieved");
cacheStorageTest(`[${environment}] none => isolate-and-require-corp`,
  context_none,
  context_require_corp,
  "",
  "include",
  "error");
cacheStorageTest(`[${environment}] none => isolate-and-require-corp (omit)`,
  context_none,
  context_require_corp,
  "",
  "include",
  "error");
cacheStorageTest(`[${environment}] none => isolate-and-require-corp + CORP`,
  context_none,
  context_require_corp,
  corp_cross_origin,
  "include",
  "retrieved");

cacheStorageTest(`[${environment}] isolate-and-credentialless => none`,
  context_credentialless,
  context_none,
  "",
  "include",
  "retrieved");
cacheStorageTest(`[${environment}] isolate-and-credentialless => isolate-and-credentialless`,
  context_credentialless,
  context_credentialless,
  "",
  "include",
  "retrieved");
cacheStorageTest(`[${environment}] isolate-and-credentialless => isolate-and-require-corp`,
  context_credentialless,
  context_require_corp,
  "",
  "include",
  "error");
cacheStorageTest(`[${environment}] isolate-and-credentialless => isolate-and-require-corp + CORP`,
  context_credentialless,
  context_require_corp,
  corp_cross_origin,
  "include",
  "retrieved");

cacheStorageTest(`[${environment}] isolate-and-require-corp => none`,
  context_require_corp,
  context_none,
  corp_cross_origin,
  "include",
  "retrieved");
cacheStorageTest(`[${environment}] isolate-and-require-corp => isolate-and-credentialless`,
  context_require_corp,
  context_credentialless,
  corp_cross_origin,
  "include",
  "retrieved");
cacheStorageTest(`[${environment}] isolate-and-require-corp => isolate-and-require-corp`,
  context_require_corp,
  context_require_corp,
  corp_cross_origin,
  "include",
  "retrieved");
