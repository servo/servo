const testSetupPolicy = trustedTypes.createPolicy("p", { createScriptURL: s => s });
importScripts(testSetupPolicy.createScriptURL("/resources/testharness.js"));

trustedTypes.createPolicy('default', {createScript: s => s});

promise_test(t => {
  globalThis.evalScriptRan = false;
  const violation = new EventWatcher(t, self, 'securitypolicyviolation')
      .wait_for('securitypolicyviolation')
      .then(e => {
        assert_equals(e.effectiveDirective, 'script-src');
        assert_equals(e.blockedURI, 'eval');
        assert_false(globalThis.evalScriptRan);
      });
  assert_throws_js(Error, () => {
    try {
      eval("globalThis.evalScriptRan = true;");
    } catch (e) {
      throw new Error();
    }
  });
  return violation;
}, "Scripts injected via direct `eval` are not allowed with `trusted-types-eval` without `require-trusted-types-for 'script'` (DedicatedWorker).");

promise_test(t => {
  globalThis.setTimeoutScriptRan = false;
  const violation = new EventWatcher(t, self, 'securitypolicyviolation')
      .wait_for('securitypolicyviolation')
      .then(e => {
        assert_equals(e.effectiveDirective, 'script-src');
        assert_equals(e.blockedURI, 'eval');
        return new Promise(resolve => t.step_timeout(resolve, 0));
      })
      .then(() => assert_false(globalThis.setTimeoutScriptRan));
  const id = setTimeout("globalThis.setTimeoutScriptRan = true;", 0);
  t.add_cleanup(() => clearTimeout(id));
  return violation;
}, "Scripts injected via `setTimeout` are not allowed with `trusted-types-eval` without `require-trusted-types-for 'script'` (DedicatedWorker).");

promise_test(t => {
  globalThis.setIntervalScriptRan = false;
  const violation = new EventWatcher(t, self, 'securitypolicyviolation')
      .wait_for('securitypolicyviolation')
      .then(e => {
        assert_equals(e.effectiveDirective, 'script-src');
        assert_equals(e.blockedURI, 'eval');
        return new Promise(resolve => t.step_timeout(resolve, 0));
      })
      .then(() => assert_false(globalThis.setIntervalScriptRan));
  const id = setInterval("globalThis.setIntervalScriptRan = true;", 0);
  t.add_cleanup(() => clearInterval(id));
  return violation;
}, "Scripts injected via `setInterval` are not allowed with `trusted-types-eval` without `require-trusted-types-for 'script'` (DedicatedWorker).");

done();
