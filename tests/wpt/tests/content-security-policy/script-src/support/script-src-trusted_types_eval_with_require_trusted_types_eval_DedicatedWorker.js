const testSetupPolicy = trustedTypes.createPolicy("p", { createScriptURL: s => s });
importScripts(testSetupPolicy.createScriptURL("/resources/testharness.js"));

trustedTypes.createPolicy('default', {createScript: s => s});

promise_test(t => {
  globalThis.evalScriptRan = false;
  const violation = new EventWatcher(t, self, 'securitypolicyviolation')
      .wait_for('securitypolicyviolation')
      .then(() => assert_unreached('Unexpected securitypolicyviolation'));
  eval("globalThis.evalScriptRan = true;");
  return Promise.race([
    violation,
    new Promise(resolve => t.step_timeout(resolve, 0)),
  ]).then(() => assert_true(globalThis.evalScriptRan));
}, "Script injected via direct `eval` is allowed with `trusted-types-eval` and `require-trusted-types-for 'script'` (Dedicated Worker).");

promise_test(t => {
  globalThis.setTimeoutScriptRan = false;
  const violation = new EventWatcher(t, self, 'securitypolicyviolation')
      .wait_for('securitypolicyviolation')
      .then(() => assert_unreached('Unexpected securitypolicyviolation'));
  const id = setTimeout("globalThis.setTimeoutScriptRan = true;", 0);
  t.add_cleanup(() => clearTimeout(id));
  return Promise.race([
    violation,
    new Promise(resolve => t.step_timeout(resolve, 0)),
  ]).then(() => assert_true(globalThis.setTimeoutScriptRan));
}, "Script injected via `setTimeout` is allowed with `trusted-types-eval` and `require-trusted-types-for 'script'` (Dedicated Worker).");

promise_test(t => {
  globalThis.setIntervalScriptRan = false;
  const violation = new EventWatcher(t, self, 'securitypolicyviolation')
      .wait_for('securitypolicyviolation')
      .then(() => assert_unreached('Unexpected securitypolicyviolation'));
  const id = setInterval("globalThis.setIntervalScriptRan = true;", 0);
  t.add_cleanup(() => clearInterval(id));
  return Promise.race([
    violation,
    new Promise(resolve => t.step_timeout(resolve, 0)),
  ]).then(() => assert_true(globalThis.setIntervalScriptRan));
}, "Script injected via `setInterval` is allowed with `trusted-types-eval` and `require-trusted-types-for 'script'` (Dedicated Worker).");

done();
