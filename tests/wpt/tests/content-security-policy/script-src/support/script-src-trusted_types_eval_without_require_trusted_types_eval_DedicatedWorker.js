const testSetupPolicy = trustedTypes.createPolicy("p", { createScriptURL: s => s });
importScripts(testSetupPolicy.createScriptURL("/resources/testharness.js"));

trustedTypes.createPolicy('default', {createScript: s => s});

var evalScriptRan = false;

async_test(function(t) {
  var eventHandler = t.step_func_done(function(e) {
    assert_false(evalScriptRan);
    assert_equals(e.effectiveDirective, 'script-src');
    assert_equals(e.blockedURI, 'eval');
  });
  self.addEventListener('securitypolicyviolation', eventHandler);
  t.add_cleanup(() => {
    self.removeEventListener('securitypolicyviolation', eventHandler);
  });
  assert_throws_js(Error,
    function() {
      try {
        eval("evalScriptRan = true;");
      } catch (e) {
        throw new Error();
    }
  });
}, "Scripts injected via direct `eval` are not allowed with `trusted-types-eval` without `require-trusted-types-for 'script'` (DedicatedWorker).");

done();
