// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/notifications/resources/helpers.js

// NOTE:
// We are not testing success cases here as doing so will try creating external network
// connection, which is not allowed by all browser test environments.
// (e.g. Gecko explicitly disables push service for testing environment.)
// Ideally we should have WPT-specific mock server in this case. See also
// https://github.com/w3c/push-api/issues/365.

promise_setup(async () => {
  // The spec does not enforce validation order and implementations
  // indeed check other things before checking applicationServerKey.

  // Get the permission because Firefox checks it before key validation.
  // (The permission test is done in permission.https.html.)
  await trySettingPermission("granted");
  // Get the active service worker because Chrome checks it before key validation
  registration = await getActiveServiceWorker("noop-sw.js");
});

promise_test(async (t) => {
  await promise_rejects_dom(
    t,
    "InvalidAccessError",
    registration.pushManager.subscribe({ applicationServerKey: "" }),
  );
}, "Reject empty string applicationServerKey");

promise_test(async (t) => {
  await promise_rejects_dom(
    t,
    "InvalidAccessError",
    registration.pushManager.subscribe({ applicationServerKey: new ArrayBuffer(0) }),
  );
}, "Reject empty ArrayBuffer applicationServerKey");

promise_test(async (t) => {
  await promise_rejects_dom(
    t,
    "InvalidAccessError",
    registration.pushManager.subscribe({ applicationServerKey: new Uint8Array(0) }),
  );
}, "Reject empty Uint8Array applicationServerKey");

promise_test(async (t) => {
  await promise_rejects_dom(
    t,
    "InvalidAccessError",
    registration.pushManager.subscribe({ applicationServerKey: new Uint8Array([1, 2, 3]) }),
  );
}, "Reject a key that is not a valid point on P-256 curve");

promise_test(async (t) => {
  await promise_rejects_dom(
    t,
    "InvalidCharacterError",
    registration.pushManager.subscribe({ applicationServerKey: "!@#$^&*" }),
  );
}, "Reject a string key that can't be decoded by base64url");
