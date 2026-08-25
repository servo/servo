// META: timeout=long
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js

// Regression test for: https://crbug.com/1256822.
//
// From a sandboxed iframe allowing popups, scripts, and same-origin. Open a
// popup using the WindowProxy of a new iframe that is still on the initial
// empty document. Check that the sandbox flags are properly inherited.

// Return true if the execution context is sandboxed.
const isSandboxed = () => {
  try {
    // Setting document.domain in sandboxed document throw errors.
    document.domain = document.domain;
    return false;
  } catch (error) {
    return true;
  }
}

promise_test(async test => {
  // 1. Create a sandboxed iframe, allowing popups, same-origin and scripts.
  const iframe_token = token();
  const iframe_document = new RemoteContext(iframe_token);
  const iframe_url = remoteExecutorUrl(iframe_token);
  const iframe = document.createElement("iframe");
  iframe.sandbox = "allow-same-origin allow-scripts allow-popups";
  iframe.src = iframe_url;
  document.body.appendChild(iframe);
  assert_true(await iframe_document.execute_script(isSandboxed),
    "iframe is sandboxed");

  // 2. From the sandboxed iframe, create an empty iframe, and open a popup
  //    using it's WindowProxy. The popup must inherit sandbox flags.
  const popup_token = token();
  const popup_document = new RemoteContext(popup_token);
  const popup_url = remoteExecutorUrl(popup_token);
  iframe_document.execute_script((popup_url) => {
    let iframe = document.createElement("iframe");
    iframe.name = "iframe_name";
    document.body.appendChild(iframe);
    iframe_name.open(popup_url);
  }, [popup_url.href]);
  assert_true(await popup_document.execute_script(isSandboxed), "popup is sandboxed");
});
