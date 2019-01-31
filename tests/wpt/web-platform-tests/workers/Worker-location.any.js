// META: global=!default, dedicatedworker, sharedworker
test(() => {
  assert_equals(String(WorkerLocation), "function WorkerLocation() { [native code] }");
  assert_true(location instanceof Object);
  assert_equals(location.href, "http://web-platform.test:8001/workers/Worker-location.any.worker.js")
  assert_equals(location.origin, "http://web-platform.test:8001");
  assert_equals(location.protocol, "http:");
  assert_equals(location.host, "web-platform.test:8001");
  assert_equals(location.hostname, "web-platform.test");
  assert_equals(location.port, "8001");
  assert_equals(location.pathname, "/workers/Worker-location.any.worker.js");
  assert_equals(location.search, "");
  assert_equals(location.hash, "");
}, 'Test WorkerLocation properties.');
