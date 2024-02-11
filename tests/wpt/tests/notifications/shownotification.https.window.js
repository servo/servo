// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=resources/helpers.js
// META: script=resources/custom-data.js

"use strict";

/** @type {ServiceWorkerRegistration} */
let registration;

promise_setup(async () => {
  await test_driver.set_permission({ name: "notifications" }, "granted");
  registration = await getActiveServiceWorker("noop-sw.js");
});

promise_test(async () => {
  const notifications = await registration.getNotifications();
  assert_equals(notifications.length, 0, "Should return zero notification");
}, "fetching no notifications");

promise_test(async t => {
  t.add_cleanup(closeAllNotifications);
  await registration.showNotification("");
  const notifications = await registration.getNotifications();
  assert_equals(notifications.length, 1, "Should return one notification");
  assert_equals(notifications[0].title, "", "Should return an empty title");
}, "fetching notification with an empty title");

promise_test(async t => {
  t.add_cleanup(closeAllNotifications);
  await Promise.all([
    registration.showNotification("thunder", { tag: "fire" }),
    registration.showNotification("bird", { tag: "fox" }),
    registration.showNotification("supernova", { tag: "quantum" }),
  ]);
  const notifications = await registration.getNotifications({ tag: "quantum" });
  assert_equals(
    notifications.length,
    1,
    "Should return only the matching notification"
  );
  assert_equals(notifications[0].title, "supernova", "title should match");
  assert_equals(notifications[0].tag, "quantum", "tag should match");
}, "fetching notification by tag filter");

promise_test(async t => {
  t.add_cleanup(closeAllNotifications);
  await Promise.all([
    registration.showNotification("thunder"),
    registration.showNotification("bird"),
    registration.showNotification("supernova"),
  ]);
  const notifications = await registration.getNotifications();
  assert_equals(notifications.length, 3, "Should return three notifications");
}, "fetching multiple notifications");

// https://notifications.spec.whatwg.org/#dom-serviceworkerregistration-getnotifications
// Step 5.2: Let notifications be a list of all notifications in the list of
// notifications ... whose service worker registration is this ...
promise_test(async t => {
  t.add_cleanup(closeAllNotifications);
  const another = await navigator.serviceWorker.register("noop-sw.js", { scope: "./scope" });
  await registration.showNotification("Hello");
  const notifications = await another.getNotifications();
  assert_equals(notifications.length, 0, "Should return no notification");
}, "fetching from another registration")

// https://notifications.spec.whatwg.org/#non-persistent-notification
// A non-persistent notification is a notification without an associated
// service worker registration.
promise_test(async t => {
  t.add_cleanup(closeAllNotifications);
  const nonPersistent = new Notification("Non-persistent");
  t.add_cleanup(() => nonPersistent.close());
  await registration.showNotification("Hello");
  const notifications = await registration.getNotifications();
  assert_equals(notifications.length, 1, "Should return a notification");
  assert_equals(notifications[0].title, "Hello", "Title should match");
}, "fetching only persistent notifications")

promise_test(async t => {
  t.add_cleanup(closeAllNotifications);
  await registration.showNotification("Hello", { data: fakeCustomData });
  const notifications = await registration.getNotifications();
  assert_equals(notifications.length, 1, "Should return a notification");
  assert_custom_data(notifications[0].data);
}, "fetching a notification with custom data")
