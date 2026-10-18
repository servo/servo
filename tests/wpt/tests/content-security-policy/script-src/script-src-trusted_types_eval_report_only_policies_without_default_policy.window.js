// META: title=With report-only policies only, both the Trusted Types and the script-src checks report, and neither blocks.

// Content-Security-Policy-Report-Only: script-src 'self' 'trusted-types-eval';
// require-trusted-types-for 'script'
//
// https://w3c.github.io/webappsec-csp/#can-compile-strings: the Trusted Types
// check runs first and, since there is no default policy, reports a
// require-trusted-types-for violation (without blocking, as the policy is
// report-only). Then, since Trusted Types are not enforced,
// 'trusted-types-eval' does not apply and the script-src check reports a
// violation as well.

const kReportOnlyPolicy =
    'script-src \'self\' \'trusted-types-eval\'; require-trusted-types-for \'script\'';

// Collects the next `count` violations.
function waitForViolations(count) {
  return new Promise(resolve => {
    const violations = [];
    const handler = e => {
      violations.push(e);
      if (violations.length === count) {
        window.removeEventListener('securitypolicyviolation', handler);
        resolve(violations);
      }
    };
    window.addEventListener('securitypolicyviolation', handler);
  });
}

function checkViolations(violations) {
  for (const e of violations) {
    assert_equals(e.originalPolicy, kReportOnlyPolicy);
    assert_equals(e.disposition, 'report');
  }
  assert_array_equals(violations.map(e => e.effectiveDirective).sort(),
                      ['require-trusted-types-for', 'script-src']);
}

var evalScriptRan = false;
var indirectEvalScriptRan = false;
var functionScriptRan = false;
var setTimeoutScriptRan = false;

promise_test(async t => {
  const violations = waitForViolations(2);
  eval('evalScriptRan = true;');
  assert_true(evalScriptRan);
  checkViolations(await violations);
}, 'Direct `eval` is allowed but reported by both checks.');

promise_test(async t => {
  const violations = waitForViolations(2);
  eval?.('indirectEvalScriptRan = true;');
  assert_true(indirectEvalScriptRan);
  checkViolations(await violations);
}, 'Indirect `eval` is allowed but reported by both checks.');

promise_test(async t => {
  const violations = waitForViolations(2);
  new Function('functionScriptRan = true;')();
  assert_true(functionScriptRan);
  checkViolations(await violations);
}, '`new Function` is allowed but reported by both checks.');

promise_test(async t => {
  const violations = waitForViolations(2);
  setTimeout('setTimeoutScriptRan = true;', 0);
  checkViolations(await violations);
  await new Promise(resolve => step_timeout(resolve, 0));
  assert_true(setTimeoutScriptRan);
}, '`setTimeout` is allowed but reported by both checks.');
