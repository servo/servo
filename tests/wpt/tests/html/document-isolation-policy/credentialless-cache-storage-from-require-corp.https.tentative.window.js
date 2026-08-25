// META: timeout=long
// META: variant=?document
// META: variant=?dedicated_worker
// META: variant=?shared_worker
// META: variant=?service_worker
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=./resources/common.js

// Fetch a resource and store it into CacheStorage from |storer| context. Then
// check if it can be retrieved via CacheStorage.match from |retriever| context.
const cacheStorageTest = (
  description,
  dip_storer,
  dip_retriever,
  resource_headers,
  request_credential_mode,
  expectation
) => {
  promise_test(async test => {
    const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;
    const resource_url = cross_origin + "/common/square.png?pipe=" + resource_headers;

    // Create the storer and retriever contexts.
    const storage_token = await getTokenFromEnvironment(test, environment, dip_storer);
    const storage_context = new RemoteContext(storage_token);
    const retriever_token = await getTokenFromEnvironment(test, environment, dip_retriever);
    const retriever_context = new RemoteContext(retriever_token);

    // Fetch a request from the storer. Store the opaque response into
    // CacheStorage.
    const stored = await storage_context.execute_script(
      async (url, credential_mode) => {
        const cache = await caches.open('v1');
        const fetch_request = new Request(url, {
          mode: 'no-cors',
          credentials: credential_mode
        });
        const fetch_response = await fetch(fetch_request);
        await cache.put(fetch_request, fetch_response);
        return true;
      }, [resource_url, request_credential_mode]);
    assert_equals(stored, true);

    // Retrieved it from |retriever|.
    const was_retrieved = await retriever_context.execute_script(
      async (url) => {
        const cache = await caches.open('v1');
         try {
           const response = await cache.match(url);
           return "retrieved";
         } catch (error) {
           return "error";
         }
      }, [resource_url]);
    assert_equals(was_retrieved, expectation);
  }, description);
};

// Execute the same set of tests for every type of execution contexts:
// Documents, DedicatedWorkers, SharedWorkers, and ServiceWorkers. The results
// should be independent of the context.
const environment = location.search.substr(1);

cacheStorageTest(`[${environment}] isolate-and-require-corp => none`,
  dip_require_corp,
  dip_none,
  corp_cross_origin,
  "include",
  "retrieved");
cacheStorageTest(`[${environment}] isolate-and-require-corp => isolate-and-credentialless`,
  dip_require_corp,
  dip_credentialless,
  corp_cross_origin,
  "include",
  "retrieved");
cacheStorageTest(`[${environment}] isolate-and-require-corp => isolate-and-require-corp`,
  dip_require_corp,
  dip_require_corp,
  corp_cross_origin,
  "include",
  "retrieved");
