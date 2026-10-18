// META: title=Eval is allowed if every policy allows it, via `trusted-types-eval`, `unsafe-eval`, or by not restricting scripts.

// Policies served (see .headers file; multiple policies are comma-separated in
// a single header, since only the last header with a given name is kept for
// auto-generated tests):
//   script-src 'self' 'trusted-types-eval'; require-trusted-types-for 'script'
//   script-src 'self' 'trusted-types-eval'
//   script-src 'self' 'unsafe-eval'
//   img-src 'none'
//   Report-Only: script-src 'self' 'trusted-types-eval'
// Since Trusted Types are enforced, the report-only policy must not report a
// violation either.

trustedTypes.createPolicy('default', {createScript: s => s});

var evalScriptRan = false;
var indirectEvalScriptRan = false;
var functionScriptRan = false;
var setTimeoutScriptRan = false;

function failOnViolation(t) {
  const handler = t.unreached_func('No CSP violation should fire.');
  window.addEventListener('securitypolicyviolation', handler);
  t.add_cleanup(() => {
    window.removeEventListener('securitypolicyviolation', handler);
  });
}

// Violation events are dispatched asynchronously, so give them a chance to
// fire before completing each test.
function flushViolations() {
  return new Promise(resolve => step_timeout(resolve, 10));
}

promise_test(async t => {
  failOnViolation(t);
  eval('evalScriptRan = true;');
  assert_true(evalScriptRan);
  await flushViolations();
}, 'Direct `eval` is allowed when every policy allows it.');

promise_test(async t => {
  failOnViolation(t);
  eval?.('indirectEvalScriptRan = true;');
  assert_true(indirectEvalScriptRan);
  await flushViolations();
}, 'Indirect `eval` is allowed when every policy allows it.');

promise_test(async t => {
  failOnViolation(t);
  new Function('functionScriptRan = true;')();
  assert_true(functionScriptRan);
  await flushViolations();
}, '`new Function` is allowed when every policy allows it.');

promise_test(async t => {
  failOnViolation(t);
  await new Promise(resolve => {
    setTimeout('setTimeoutScriptRan = true;', 0);
    step_timeout(resolve, 10);
  });
  assert_true(setTimeoutScriptRan);
  await flushViolations();
}, '`setTimeout` is allowed when every policy allows it.');
