test(() => {
  assert_false(self.isSecureContext);
}, "Lack of .https file name flag implies non-secure context");

test(() => {
  assert_equals(location.protocol, "http:");
}, "Lack of .https file name flag implies HTTP scheme");

done();
