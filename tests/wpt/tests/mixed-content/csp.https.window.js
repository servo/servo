// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js

promise_test((t) => {
  const url = `${get_host_info().HTTP_ORIGIN}/common/text-plain.txt`;
  const promise = fetch(url, { mode: "no-cors" });
  return promise_rejects_js(t, TypeError, promise, "mixed content fetch");
}, "Mixed content checks apply to fetches in sandboxed documents");

promise_test(async (t) => {
  const uuid = token();
  const context = new RemoteContext(uuid);

  const iframe = document.body.appendChild(document.createElement("iframe"));
  iframe.src = remoteExecutorUrl(uuid, { protocol: "http:" });

  const result = await Promise.race([
      context.execute_script(() => "loaded"),
      new Promise((resolve) => t.step_timeout(() => {
        resolve("timed out");
      }, 1000 /* ms */)),
  ]);
  assert_equals(result, "timed out");
}, "Mixed content checks apply to iframes in sandboxed documents");


promise_test(async (t) => {
  const uuid = token();

  const popup = window.open(remoteExecutorUrl(uuid, { protocol: "http:" }));

  const context = new RemoteContext(uuid);
  const result = await Promise.race([
      context.execute_script(() => "loaded"),
      new Promise((resolve) => t.step_timeout(() => {
        resolve("timed out");
      }, 1000 /* ms */)),
  ]);
  assert_equals(result, "timed out");
}, "Mixed content checks apply to popups in sandboxed documents");
