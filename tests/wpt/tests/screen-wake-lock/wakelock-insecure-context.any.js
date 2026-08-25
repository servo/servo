//META: title=Wake Lock API is not exposed in an insecure context

test(() => {
  assert_false("WakeLock" in self, "'WakeLock' must not be exposed");
}, "Wake Lock API is not exposed in an insecure context");
