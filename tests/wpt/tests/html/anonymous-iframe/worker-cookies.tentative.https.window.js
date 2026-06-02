// META: timeout=long
// META: variant=?worker=dedicated_worker
// META: variant=?worker=shared_worker
// META: variant=?worker=service_worker
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/html/cross-origin-embedder-policy/credentialless/resources/common.js
// META: script=./resources/common.js

// Execute the same set of tests for every type of worker.
// - DedicatedWorkers
// - SharedWorkers
// - ServiceWorkers.
const params = new URLSearchParams(document.location.search);
const worker_param = params.get("worker") || "dedicated_worker";

const cookie_key = token();
const cookie_value = "cookie_value";
const cookie_origin = get_host_info().HTTPS_REMOTE_ORIGIN;

// Create worker spawned from `context` and return its uuid.
const workerFrom = context => {
  const reply = token();
  send(context, `
    for(deps of [
      "/common/utils.js",
      "/resources/testharness.js",
      "/html/cross-origin-embedder-policy/credentialless/resources/common.js",
    ]) {
      await new Promise(resolve => {
        const script = document.createElement("script");
        script.src = deps;
        script.onload = resolve;
        document.body.appendChild(script);
      });
    }

    const worker_constructor = environments["${worker_param}"];
    const headers = "";
    const [worker, error] = worker_constructor(headers);
    send("${reply}", worker);
  `);
  return receive(reply);
};

// Set a cookie from a top-level document.
promise_test(async test => {
  await setCookie(cookie_origin, cookie_key, cookie_value);
}, "set cookies");

// Control: iframe is not credentialless. The worker can access cookies.
promise_test(async test => {
  const headers = token();
  send(await workerFrom(newIframe(cookie_origin)), `
    fetch("${showRequestHeaders(cookie_origin, headers)}");
  `);
  const cookie = parseCookies(JSON.parse(await receive(headers)));
  assert_equals(cookie[cookie_key], cookie_value)
}, "Worker spawned from normal iframe can access global cookies");

// Experiment: iframe is credentialless.
promise_test(async test => {
  const headers = token();
  send(await workerFrom(newIframeCredentialless(cookie_origin)), `
    fetch("${showRequestHeaders(cookie_origin, headers)}");
  `);
  const cookie = parseCookies(JSON.parse(await receive(headers)));
  assert_equals(cookie[cookie_key], undefined)
}, "Worker spawned from credentialless iframe can't access global cookies");
