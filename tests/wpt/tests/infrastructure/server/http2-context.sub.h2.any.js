// META: global=window,dedicatedworker,sharedworker,serviceworker
test(() => {
  assert_true(self.isSecureContext);
}, "Use of .h2. file name flag implies secure context");

test(() => {
  assert_equals(location.protocol, "https:");
}, "Use of .h2. file name flag implies HTTPS scheme");

test(() => {
  assert_equals(location.port, "{{ports[h2][0]}}");
}, "Use of .h2. file name flag implies correct port");
