// META: title=`trusted-types-eval` does not bypass Trusted Types: plain strings are still blocked without a default policy.

// https://w3c.github.io/webappsec-csp/#can-compile-strings: the Trusted Types
// check ("get trusted type compliant string") always runs first.
// 'trusted-types-eval' only stands in for 'unsafe-eval' in the subsequent
// per-policy CSP check. Without a default policy, plain strings must therefore
// be rejected, while TrustedScripts are allowed.
const policy = trustedTypes.createPolicy('p', {createScript: s => s});

function waitForViolation() {
  return new Promise(resolve => {
    window.addEventListener('securitypolicyviolation', resolve, {once: true});
  });
}

function checkTrustedTypesViolation(e) {
  assert_equals(e.effectiveDirective, 'require-trusted-types-for');
  assert_equals(e.disposition, 'enforce');
}

function failOnViolation(t) {
  const handler = t.unreached_func('No CSP violation should fire.');
  window.addEventListener('securitypolicyviolation', handler);
  t.add_cleanup(() => {
    window.removeEventListener('securitypolicyviolation', handler);
  });
}

var evalScriptRan = false;
var indirectEvalScriptRan = false;
var functionScriptRan = false;
var setTimeoutScriptRan = false;
var trustedEvalScriptRan = false;
var trustedSetTimeoutScriptRan = false;

promise_test(async t => {
  const violation = waitForViolation();
  assert_throws_js(EvalError, () => {
    eval('evalScriptRan = true;');
  });
  assert_false(evalScriptRan);
  checkTrustedTypesViolation(await violation);
}, 'Direct `eval` of a string is blocked by Trusted Types despite `trusted-types-eval`.');

promise_test(async t => {
  const violation = waitForViolation();
  assert_throws_js(EvalError, () => {
    eval?.('indirectEvalScriptRan = true;');
  });
  assert_false(indirectEvalScriptRan);
  checkTrustedTypesViolation(await violation);
}, 'Indirect `eval` of a string is blocked by Trusted Types despite `trusted-types-eval`.');

promise_test(async t => {
  const violation = waitForViolation();
  assert_throws_js(EvalError, () => {
    new Function('functionScriptRan = true;')();
  });
  assert_false(functionScriptRan);
  checkTrustedTypesViolation(await violation);
}, '`new Function` with a string is blocked by Trusted Types despite `trusted-types-eval`.');

promise_test(async t => {
  const violation = waitForViolation();
  assert_throws_js(TypeError, () => {
    setTimeout('setTimeoutScriptRan = true;', 0);
  });
  checkTrustedTypesViolation(await violation);
  await new Promise(resolve => step_timeout(resolve, 10));
  assert_false(setTimeoutScriptRan);
}, '`setTimeout` with a string is blocked by Trusted Types despite `trusted-types-eval`.');

promise_test(async t => {
  failOnViolation(t);
  eval(policy.createScript('trustedEvalScriptRan = true;'));
  assert_true(trustedEvalScriptRan);
  await new Promise(resolve => step_timeout(resolve, 10));
}, '`eval` of a TrustedScript is allowed with `trusted-types-eval`.');

promise_test(async t => {
  failOnViolation(t);
  await new Promise(resolve => {
    setTimeout(policy.createScript('trustedSetTimeoutScriptRan = true;'), 0);
    step_timeout(resolve, 10);
  });
  assert_true(trustedSetTimeoutScriptRan);
}, '`setTimeout` with a TrustedScript is allowed with `trusted-types-eval`.');
