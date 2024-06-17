// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/helpers.js

// (Cannot use `global=window,worker` because testdriver only supports window)

let registration;

promise_setup(async () => {
  await trySettingPermission("granted");
  registration = await getActiveServiceWorker("noop-sw.js");
});

const resolvedUrl = new URL("foo.png", location.href).toString();

promise_test(async t => {
  const n = new Notification("new Notification", { icon: "foo.png" });
  t.add_cleanup(() => n.close());

  assert_equals(n.icon, resolvedUrl, "should give a resolved URL");
}, "new Notification() should give a resolved icon URL");

promise_test(async t => {
  t.add_cleanup(closeAllNotifications);

  await registration.showNotification("showNotification", { icon: "foo.png" });

  let notifications = await registration.getNotifications();
  assert_equals(notifications.length, 1, "The list should include one notification");
  assert_equals(notifications[0].icon, resolvedUrl, "should give a resolved URL");
}, "getNotifications() should give a resolved icon URL");
