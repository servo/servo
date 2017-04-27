importScripts("{{location[scheme]}}://{{host}}:{{location[port]}}/resources/testharness.js");
importScripts("{{location[scheme]}}://{{host}}:{{location[port]}}/content-security-policy/support/testharness-helper.js");

var cspEventFiredInDocument = false;
// ServiceWorker and Worker
self.addEventListener("message", e => {
  if (e.data == "SecurityPolicyViolation from Document")
    cspEventFiredInDocument = true;
});
// SharedWorker
self.addEventListener("connect", c => {
  c.ports[0].addEventListener("message", m => {
    if (m.data == "SecurityPolicyViolation from Document")
      cspEventFiredInDocument = true;
  });
});

async_test(t => {
  var url = "{{location[scheme]}}://{{host}}:{{location[port]}}/content-security-policy/support/resource.py";
  assert_no_csp_event_for_url(t, url);

  fetch(url)
    .catch(t.unreached_func("Fetch should succeed."))
    .then(t.step_func_done(r => {
      assert_equals(r.status, 200);
      assert_false(cspEventFiredInDocument);
    }));
}, "No SecurityPolicyViolation event fired for successful load.");

async_test(t => {
  var url = "{{location[scheme]}}://{{domains[www2]}}:{{location[port]}}/content-security-policy/support/resource.py";
  waitUntilCSPEventForURL(t, url)
    .then(t.step_func_done(e => {
      assert_equals(e.blockedURI, url);
      assert_false(cspEventFiredInDocument);
    }));

  fetch(url)
    .then(t.unreached_func("Fetch should not succeed."))
    .catch(t.step_func(e => assert_true(e instanceof TypeError)));
}, "SecurityPolicyViolation event fired on global.");

// Worker tests need an explicit `done()`.
done();
