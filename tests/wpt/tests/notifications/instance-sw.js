importScripts("/resources/testharness.js");
importScripts("resources/helpers.js");
importScripts("resources/custom-data.js");
importScripts("instance-checks.js");

promise_setup(async () => {
  await untilActivate();
});

notification_instance_test(async t => {
  t.add_cleanup(closeAllNotifications);

  await registration.showNotification(...notification_args);

  let notifications = await registration.getNotifications();
  assert_equals(notifications.length, 1, "The list should include one notification");

  return notifications[0];
}, "getNotifications()");

// Doing this separately because this times out on Blink and GeckoView
notification_instance_test(async t => {
  t.add_cleanup(closeAllNotifications);

  await registration.showNotification(...notification_args);

  let notifications = await registration.getNotifications();
  assert_equals(notifications.length, 1, "The list should include one notification");

  notifications[0].close();
  const ev = await new Promise(resolve => addEventListener("notificationclose", resolve, { once: true }));

  return ev.notification;
}, "notificationclose");
