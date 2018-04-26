test(() => {
  assert_true(self.isSecureContext);
}, "Use of .https file name flag implies secure context");

test(() => {
  assert_equals(location.protocol, "https:");
}, "Use of .https file name flag implies HTTPS scheme");

done();
