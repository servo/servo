importScripts("{{location[server]}}/resources/testharness.js");
importScripts("{{location[server]}}/content-security-policy/support/testharness-helper.js");

let cspEventFiredInDocument = false;
self.addEventListener("message", e => {
  if (e.data == "SecurityPolicyViolation from Document") {
    cspEventFiredInDocument = true;
  }
});

async_test(t => {
  const url = new URL("{{location[server]}}/content-security-policy/support/ping.js").toString();
  const w = new Worker(url);
  w.onmessage = t.unreached_func("Ping should not be sent.");
  Promise.all([
    waitUntilCSPEventForURL(t, url)
      .then(t.step_func_done(e => {
        assert_equals(e.blockedURI, url);
        assert_equals(e.violatedDirective, "worker-src");
        assert_equals(e.effectiveDirective, "worker-src");
        assert_false(cspEventFiredInDocument, "Should not have fired event on document");
      })),
    waitUntilEvent(w, "error"),
  ]);
}, "Nested worker with worker-src is disallowed.");

done();
