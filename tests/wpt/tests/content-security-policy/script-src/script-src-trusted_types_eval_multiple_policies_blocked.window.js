// META: title=`trusted-types-eval` in one policy does not allow eval if another enforced policy blocks it.

// Policies served (see .headers file; multiple policies are comma-separated in
// a single header, since only the last header with a given name is kept for
// auto-generated tests):
//   script-src 'self' 'trusted-types-eval'; require-trusted-types-for 'script'
//   script-src 'self'

// Install a pass-through default policy so that the Trusted Types check
// succeeds, and only the per-policy CSP check is exercised.
trustedTypes.createPolicy('default', {createScript: s => s});

// https://w3c.github.io/webappsec-csp/#can-compile-strings: the
// 'trusted-types-eval' check is performed for each policy. The second policy
// contains neither 'trusted-types-eval' nor 'unsafe-eval', so it must block
// (and report) string compilation.
const kBlockingPolicy = 'script-src \'self\'';

function waitForViolation() {
  return new Promise(resolve => {
    window.addEventListener('securitypolicyviolation', resolve, {once: true});
  });
}

function checkViolation(e) {
  assert_equals(e.effectiveDirective, 'script-src');
  assert_equals(e.blockedURI, 'eval');
  assert_equals(e.originalPolicy, kBlockingPolicy);
  assert_equals(e.disposition, 'enforce');
}

var evalScriptRan = false;
var indirectEvalScriptRan = false;
var functionScriptRan = false;
var setTimeoutScriptRan = false;

promise_test(async t => {
  const violation = waitForViolation();
  assert_throws_js(EvalError, () => {
    eval('evalScriptRan = true;');
  });
  assert_false(evalScriptRan);
  checkViolation(await violation);
}, 'Direct `eval` is blocked by a second policy without `trusted-types-eval`.');

promise_test(async t => {
  const violation = waitForViolation();
  assert_throws_js(EvalError, () => {
    eval?.('indirectEvalScriptRan = true;');
  });
  assert_false(indirectEvalScriptRan);
  checkViolation(await violation);
}, 'Indirect `eval` is blocked by a second policy without `trusted-types-eval`.');

promise_test(async t => {
  const violation = waitForViolation();
  assert_throws_js(EvalError, () => {
    new Function('functionScriptRan = true;')();
  });
  assert_false(functionScriptRan);
  checkViolation(await violation);
}, '`new Function` is blocked by a second policy without `trusted-types-eval`.');

promise_test(async t => {
  const violation = waitForViolation();
  setTimeout('setTimeoutScriptRan = true;', 0);
  checkViolation(await violation);
  await new Promise(resolve => step_timeout(resolve, 10));
  assert_false(setTimeoutScriptRan);
}, '`setTimeout` is blocked by a second policy without `trusted-types-eval`.');
