// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=resources/helpers.js

"use strict";

/** @type {ServiceWorkerRegistration} */
let registration;

promise_setup(async () => {
  registration = await prepareActiveServiceWorker("noop-sw.js");
  await trySettingPermission("granted");
});

promise_test(async (t) => {
  t.add_cleanup(closeAllNotifications);

  await registration.showNotification("foo");
  await registration.unregister();
  registration = await prepareActiveServiceWorker("noop-sw.js");
  const notifications = await registration.getNotifications();

  // The spec says notifications should be associated with service worker registration
  // and thus unregistering should dissociate the notification.
  //
  // (Step 5.2 of https://notifications.spec.whatwg.org/#dom-serviceworkerregistration-getnotifications)
  // > Let notifications be a list of all notifications in the list of notifications whose origin
  // > is same origin with origin, whose service worker registration is this, and whose tag, if tag
  // > is not the empty string, is tag.
  assert_equals(notifications.length, 0, "Should return zero notification");
}, "A new SW registration gets no previous notification");

promise_test(async (t) => {
  await registration.showNotification("foo");
  await registration.unregister();
  const notifications = await registration.getNotifications();
  assert_equals(notifications.length, 0, "Should return zero notification");
}, "An unregistered SW registration gets no previous notification");
