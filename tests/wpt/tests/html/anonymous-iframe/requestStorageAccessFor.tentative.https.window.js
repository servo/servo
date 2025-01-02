// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/html/cross-origin-embedder-policy/credentialless/resources/common.js
// META: script=./resources/common.js

setup(() => {
  assert_implements(document.requestStorageAccessFor,
    "requestStorageAccessFor is not supported.");
})

const requestStorageAccessFor = (iframe, origin) => {
  const reply = token();
  send(iframe, `
    try {
      await document.requestStorageAccessFor("${origin}");
      send("${reply}", "success");
    } catch {
      send("${reply}", "failed");
    }
  `);
  return receive(reply);
}

promise_test(async test => {
  const same_origin = window.origin;
  const iframe = newIframeCredentialless(same_origin);
  assert_equals(await requestStorageAccessFor(iframe, same_origin), "failed");
}, "Same-origin credentialless iframe can't request storage access");

promise_test(async test => {
  const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;
  const iframe = newIframeCredentialless(cross_origin);
  assert_equals(await requestStorageAccessFor(iframe, cross_origin), "failed");
}, "Cross-origin credentialless iframe can't request storage access");
