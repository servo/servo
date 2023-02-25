// META: timeout=long
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
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

  // One window with COEP:none. (control)
  const w_control_token = token();
  const w_control_url = same_origin + executor_path +
    coep_none + `&uuid=${w_control_token}`
  const w_control = window.open(w_control_url);
  add_completion_callback(() => w_control.close());

  // One window with COEP:credentialless. (experiment)
  const w_credentialless_token = token();
  const w_credentialless_url = same_origin + executor_path +
    coep_credentialless + `&uuid=${w_credentialless_token}`;
  const w_credentialless = window.open(w_credentialless_url);
  add_completion_callback(() => w_credentialless.close());

  let GetCookie = (response) => {
    const headers_credentialless = JSON.parse(response);
    return parseCookies(headers_credentialless)[cookie_key];
  }

  const dedicatedWorkerTest = function(
    description, origin, coep_for_worker,
    expected_cookies_control,
    expected_cookies_credentialless) {
    promise_test_parallel(async t => {
      // Create workers for both window.
      const worker_token_1 = token();
      const worker_token_2 = token();

      // Used to check for errors creating the DedicatedWorker.
      const worker_error_1 = token();
      const worker_error_2 = token();

      const w_worker_src_1 = same_origin + executor_worker_path +
        coep_for_worker + `&uuid=${worker_token_1}`;
      send(w_control_token, `
        new Worker("${w_worker_src_1}", {});
        worker.onerror = () => {
          send("${worker_error_1}", "Worker blocked");
        }
      `);

      const w_worker_src_2 = same_origin + executor_worker_path +
        coep_for_worker + `&uuid=${worker_token_2}`;
      send(w_credentialless_token, `
        const worker = new Worker("${w_worker_src_2}", {});
        worker.onerror = () => {
          send("${worker_error_2}", "Worker blocked");
        }
      `);

      // Fetch resources with the workers.
      const request_token_1 = token();
      const request_token_2 = token();
      const request_url_1 = showRequestHeaders(origin, request_token_1);
      const request_url_2 = showRequestHeaders(origin, request_token_2);

      send(worker_token_1, `
        fetch("${request_url_1}", {mode: 'no-cors', credentials: 'include'})
      `);
      send(worker_token_2, `
        fetch("${request_url_2}", {mode: 'no-cors', credentials: 'include'});
      `);

      const response_control = await Promise.race([
        receive(worker_error_1),
        receive(request_token_1).then(GetCookie)
      ]);
      assert_equals(response_control,
        expected_cookies_control,
        "coep:none => ");

      const response_credentialless = await Promise.race([
        receive(worker_error_2),
        receive(request_token_2).then(GetCookie)
      ]);
      assert_equals(response_credentialless,
        expected_cookies_credentialless,
        "coep:credentialless => ");
    }, `fetch ${description}`)
  };

  dedicatedWorkerTest("same-origin + credentialless worker",
    same_origin, coep_credentialless,
    cookie_same_origin,
    cookie_same_origin);

  dedicatedWorkerTest("same-origin + require_corp worker",
    same_origin, coep_require_corp,
    cookie_same_origin,
    cookie_same_origin);

  dedicatedWorkerTest("same-origin",
    same_origin, coep_none,
    cookie_same_origin,
    "Worker blocked");

  dedicatedWorkerTest("cross-origin",
    cross_origin, coep_none,
    cookie_cross_origin,
    "Worker blocked" // Owner's policy is credentialles, so we can't
                     // create a worker with coep_none.
  );

  dedicatedWorkerTest("cross-origin + credentialless worker",
    cross_origin, coep_credentialless,
    undefined, // Worker created successfully with credentialless, and fetch doesn't get credentials
    undefined // Worker created successfully with credentialless, and fetch doesn't get credentials
  );

  dedicatedWorkerTest("cross-origin + require_corp worker",
    cross_origin, coep_require_corp,
    cookie_cross_origin,
    cookie_cross_origin // The worker's policy is require_corp and doing a
                        // fetch within it has nothing to do with the Owner's policy.
  );
})
