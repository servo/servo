// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=resources/helpers.js

"use strict";

/** @type {ServiceWorkerRegistration} */
let registration;

promise_setup(async () => {
  await trySettingPermission("prompt");
  registration = await getActiveServiceWorker("noop-sw.js");
  await closeAllNotifications();
});

promise_test(async t => {
  t.add_cleanup(closeAllNotifications);

  await promise_rejects_js(t, TypeError, registration.showNotification(""), "Should throw TypeError");
  const notifications = await registration.getNotifications();
  assert_equals(notifications.length, 0, "Should return zero notification");
}, "showNotificaiton should not be listed with permission=default")
