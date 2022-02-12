// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js

// Regression test for https://crbug.com/1262203
//
// A cross-origin document initiates a same-document navigation. This navigation
// is subject to CSP:frame-src 'none', but this doesn't apply, since it's a
// same-document navigation. This test checks this doesn't lead to a crash.

promise_test(async test => {
  const child_token = token();
  const child = new RemoteContext(child_token);
  const iframe = document.createElement("iframe");
  iframe.src = get_host_info().REMOTE_ORIGIN +
      "/content-security-policy/frame-src/support/executor.html" +
      `?uuid=${child_token}`;
  document.body.appendChild(iframe);

  // Install a promise waiting for a same-document navigation to happen in the
  // child.
  await child.execute_script(() => {
    window.sameDocumentNavigation = new Promise(resolve => {
      window.addEventListener("popstate", resolve);
    });
  });

  // Append a new CSP, disallowing new iframe navigations.
  const meta = document.createElement("meta");
  meta.httpEquiv = "Content-Security-Policy";
  meta.content = "frame-src 'none'";
  document.head.appendChild(meta);

  document.addEventListener(
      "securitypolicyviolation",
      test.unreached_func("same-document navigations aren't subject to CSP"));

  // Create a same-document navigation, inititated cross-origin in the iframe.
  // It must not be blocked by the CSP above.
  iframe.src += "#foo";

  // Make sure the navigation succeeded and was indeed a same-document one:
  await child.execute_script(() => sameDocumentNavigation);
  assert_equals(await child.execute_script(() => location.href), iframe.src);
})
