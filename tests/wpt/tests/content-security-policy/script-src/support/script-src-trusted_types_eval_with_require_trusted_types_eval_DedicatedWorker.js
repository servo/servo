const testSetupPolicy = trustedTypes.createPolicy("p", { createScriptURL: s => s });
importScripts(testSetupPolicy.createScriptURL("/resources/testharness.js"));

trustedTypes.createPolicy('default', {createScript: s => s});

var evalScriptRan = false;

async_test(function(t) {
  var eventHandler = t.unreached_func('No CSP violation report has fired.');
  self.addEventListener('securitypolicyviolation', eventHandler);
  t.add_cleanup(() => {
    self.removeEventListener('securitypolicyviolation', eventHandler);
  });
  try {
    eval("evalScriptRan = true;");
  } catch (e) {
    assert_unreached("`eval` should be allowed with `trusted-types-eval` and `require-trusted-types-for 'script'`.");
  }
  assert_true(evalScriptRan);
  t.done();
}, "Script injected via direct `eval` is allowed with `trusted-types-eval` and `require-trusted-types-for 'script'` (Dedicated Worker).");

done();
