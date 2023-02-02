// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

"use strict";

/** @type {ServiceWorkerRegistration} */
let registration;

function reset() {
  return navigator.serviceWorker.getRegistrations().then(registrations => {
    return Promise.all(registrations.map(r => r.unregister()));
  });
}

async function registerSw() {
  await reset();
  const reg = await navigator.serviceWorker.register("noop-sw.js");
  await navigator.serviceWorker.ready;
  return reg;
}

async function cleanup() {
  for (const n of await registration.getNotifications()) {
    n.close();
  }
}

promise_setup(async () => {
  await test_driver.set_permission({ name: "notifications" }, "granted");
  registration = await registerSw();
});

promise_test(async () => {
  const notifications = await registration.getNotifications();
  assert_equals(notifications.length, 0, "Should return zero notification");
}, "fetching no notifications");

promise_test(async t => {
  t.add_cleanup(cleanup);
  await registration.showNotification("");
  const notifications = await registration.getNotifications();
  assert_equals(notifications.length, 1, "Should return one notification");
  assert_equals(notifications[0].title, "", "Should return an empty title");
}, "fetching notification with an empty title");

promise_test(async t => {
  t.add_cleanup(cleanup);
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
  t.add_cleanup(cleanup);
  await Promise.all([
    registration.showNotification("thunder"),
    registration.showNotification("bird"),
    registration.showNotification("supernova"),
  ]);
  const notifications = await registration.getNotifications();
  assert_equals(notifications.length, 3, "Should return three notifications");
}, "fetching multiple notifications");
