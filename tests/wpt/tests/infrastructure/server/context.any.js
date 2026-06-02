// META: global=window,dedicatedworker,sharedworker,serviceworker,dedicatedworker-module,sharedworker-module,serviceworker-module
test(t => {
  // Test for object that's only exposed in serviceworker
  if (self.clients) {
      assert_true(self.isSecureContext);
      assert_equals(location.protocol, "https:");
  } else {
      assert_false(self.isSecureContext);
      assert_equals(location.protocol, "http:");
  }
});

done();
