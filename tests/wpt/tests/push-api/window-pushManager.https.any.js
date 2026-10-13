// META: global=window-module
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/notifications/resources/helpers.js

// Tests for window.pushManager
// https://w3c.github.io/push-api/#extensions-to-the-serviceworkerregistration-interface

import { pushMessage } from "/push-api/resources/helpers.js"

let windowSubscription;
promise_setup(async () => {
  await trySettingPermission("granted");
  windowSubscription = await pushManager.subscribe();
  add_completion_callback(() => windowSubscription.unsubscribe());
});

promise_test(async t => {
  const registration = await prepareActiveServiceWorkerForTest(t, "push-sw.js", {scope: "/"});
  let swSubscription = await registration.pushManager.getSubscription();
  assert_object_equals(swSubscription.toJSON(), windowSubscription.toJSON(),
    "getSubscription() for service worker should return the window subscription.");
  const {promise, resolve} = Promise.withResolvers();
  navigator.serviceWorker.addEventListener("message", e => {
    resolve(e.data.data.text);
  }, {once: true});
  await pushMessage(windowSubscription, { message: "Hello" });
  const pushText = await promise;
  assert_equals(pushText, "Hello",
    "Service worker should get push event when push message is sent through window.pushManager.");
}, "Using window.pushManager when there is a service worker with scope /");

promise_test(async t => {
  const registration = await prepareActiveServiceWorkerForTest(t, "push-sw.js");
  assert_equals(await registration.pushManager.getSubscription(), null,
    "getSubscription() should return null if only window has a push subscription.");
  const swSubscription = await registration.pushManager.subscribe();
  // https://w3c.github.io/push-api/#push-subscription
  // "A push endpoint MUST uniquely identify the push subscription."
  assert_not_equals(swSubscription.endpoint, windowSubscription.endpoint,
    "Service worker subscription should be different from window subscription.");
}, "Using window.pushManager when there is a service worker with scope NOT equal to /");
