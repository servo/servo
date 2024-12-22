// META: global=window-module
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/notifications/resources/helpers.js

import { encrypt } from "./resources/helpers.js"

let registration;

promise_setup(async () => {
  registration = await getActiveServiceWorker("push-sw.js");
});

async function subscribeWithoutKey() {
  try {
    return await registration.pushManager.subscribe();
  } catch (err) {
    if (err.name === "NotSupportedError") {
      // happens if and only if applicationServerKey omission is disallowed,
      // which is permitted per the spec. Throwing OptionalFeatureUnsupportedError marks the
      // result as PRECONDITION_FAILED.
      //
      // https://w3c.github.io/push-api/#subscribe-method
      // If the options argument does not include a non-null value for the applicationServerKey
      // member, and the push service requires one to be given, queue a global task on the
      // networking task source using global to reject promise with a "NotSupportedError"
      // DOMException.
      throw new OptionalFeatureUnsupportedError(description);
    } else {
      throw err;
    }
  }
}

promise_test(async (t) => {
  await trySettingPermission("granted")
  const subscription = await subscribeWithoutKey();
  t.add_cleanup(() => subscription.unsubscribe());

  await fetch(subscription.endpoint, {
    method: "post",
    headers: {
      TTL: 15
    }
  });

  const { data } = await new Promise(r => navigator.serviceWorker.addEventListener("message", r, { once: true }));
  assert_equals(data.constructor, "PushEvent");
}, "Posting to the push endpoint should fire push event on the service worker");

promise_test(async (t) => {
  await trySettingPermission("granted")
  const subscription = await registration.pushManager.subscribe();
  t.add_cleanup(() => subscription.unsubscribe());

  const { body, headers } = await encrypt(
    new TextEncoder().encode("Hello"),
    subscription.getKey("p256dh"),
    subscription.getKey("auth")
  );

  await fetch(subscription.endpoint, {
    method: "post",
    body,
    headers,
  });

  const { data } = await new Promise(r => navigator.serviceWorker.addEventListener("message", r, { once: true }));
  assert_equals(data.constructor, "PushEvent");
  assert_equals(new TextDecoder().decode(data.data), "Hello");

}, "Posting to the push endpoint with encrypted data should fire push event on the service worker");
