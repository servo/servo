// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=resources/helpers.js

"use strict";

/** @type {ServiceWorkerRegistration} */
let registration;

promise_setup(async () => {
  registration = await getActiveServiceWorker("noop-sw.js");
});

promise_test(async (t) => {
  t.add_cleanup(closeAllNotifications);

  try {
    await test_driver.set_permission({ name: "notifications" }, "prompt");
  } catch {
    // Not all implementations support this yet, but it may already be "prompt" to be able to continue
  }

  assert_equals(Notification.permission, "default", "Should have the default permission to continue");

  await promise_rejects_js(t, TypeError, registration.showNotification(""), "Should throw TypeError");
  const notifications = await registration.getNotifications();
  assert_equals(notifications.length, 0, "Should return zero notification");
}, "showNotificaiton should not be listed with permission=default")
