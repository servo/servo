// META: timeout=long
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=./resources/common.js

const same_origin = get_host_info().HTTPS_ORIGIN;
const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;
const cookie_key = "credentialless_dedicated_worker";
const cookie_same_origin = "same_origin";
const cookie_cross_origin = "cross_origin";

promise_test(async test => {

  await Promise.all([
    setCookie(same_origin, cookie_key, cookie_same_origin +
      cookie_same_site_none),
    setCookie(cross_origin, cookie_key, cookie_cross_origin +
      cookie_same_site_none),
  ]);

  let GetCookie = (response) => {
    const headers_credentialless = JSON.parse(response);
    return parseCookies(headers_credentialless)[cookie_key];
  }

  async function fetchInRemoteContext(ctx, request_url) {
    // The fail might fail in when a DedicatedWorker with DIP
    // isolate-and-require-corp tries to fetch a cross-origin resource. Silently
    // catch the error as we're only interested in whether the cookies were sent
    // with the fetch in the first place.
    try {
    await ctx.execute_script(
        async (url) => {
          await fetch(url, {mode: 'no-cors', credentials: 'include'});
        }, [request_url]);
    } catch(error) {}
  }

  const dedicatedWorkerTest = function(
    description, origin, dip_for_worker,
    expected_cookies) {
    promise_test_parallel(async t => {
      // Create one iframe with the specified DIP isolate-and-credentialless.
      // Then start a DedicatedWorker. The DedicatedWorker will inherit the DIP
      // of its creator.
      const worker = await createDedicatedWorkerContext(test, same_origin, dip_for_worker);
      const worker_context = new RemoteContext(worker[0]);

      // Fetch resources with the worker.
      const request_token = token();
      const request_url = showRequestHeaders(origin, request_token);

      await fetchInRemoteContext(worker_context, request_url);
      const response_worker = await receive(request_token).then(GetCookie);
      assert_equals(response_worker,
        expected_cookies,
        "dip => ");
    }, `fetch ${description}`)
  };

  dedicatedWorkerTest("same-origin + credentialless worker",
    same_origin, dip_credentialless,
    cookie_same_origin);

  dedicatedWorkerTest("same-origin + require_corp worker",
    same_origin, dip_require_corp,
    cookie_same_origin);

  dedicatedWorkerTest("cross-origin + credentialless worker",
    cross_origin, dip_credentialless,
    undefined // Worker created successfully with credentialless, and fetch doesn't get credentials
  );

  dedicatedWorkerTest("cross-origin + require_corp worker",
    cross_origin, dip_require_corp,
    cookie_cross_origin // The worker's policy is require_corp, so the resource will be requested with cookies
                        // but the load will fail because the response does not
                        // have CORP cross-origin.
  );
})
