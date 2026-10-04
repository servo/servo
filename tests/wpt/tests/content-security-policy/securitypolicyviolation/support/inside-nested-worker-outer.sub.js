importScripts(
    '{{location[scheme]}}://{{host}}:{{location[port]}}/resources/testharness.js');
importScripts(
    '{{location[scheme]}}://{{host}}:{{location[port]}}/content-security-policy/support/testharness-helper.js');

// Outer worker that creates an inner worker whose top-level script contains a
// nested static import producing a CSP violation. The 'securitypolicyviolation'
// event should be dispatched on the outer worker. See
// https://w3c.github.io/webappsec-csp/#report-violation.
async_test(t => {
  const blockedURL =
      '{{location[scheme]}}://{{domains[www1]}}:{{location[port]}}/content-security-policy/support/ping.js';

  waitUntilCSPEventForURL(t, blockedURL).then(t.step_func_done(e => {
    assert_equals(e.blockedURI, blockedURL);
  }));

  const innerWorkerURL =
      '{{location[scheme]}}://{{host}}:{{location[port]}}/content-security-policy/securitypolicyviolation/support/inside-nested-worker-inner.sub.js';
  new Worker(innerWorkerURL, {type: 'module'});
}, 'CSP violation in nested worker fires securitypolicyviolation event on outer worker.');

done();
