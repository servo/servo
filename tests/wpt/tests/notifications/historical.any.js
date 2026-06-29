test(() => {
  assert_equals(Notification.get, undefined);
}, "Notification.get is obsolete");

test(() => {
  assert_false("vibrate" in Notification.prototype);
}, "Notification.prototype.vibrate is obsolete");
