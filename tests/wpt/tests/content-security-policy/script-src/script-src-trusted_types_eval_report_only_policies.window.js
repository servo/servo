// META: title=`trusted-types-eval` has no effect when Trusted Types are only report-only.

// Content-Security-Policy-Report-Only: script-src 'self' 'trusted-types-eval';
// require-trusted-types-for 'script'
//
// https://w3c.github.io/webappsec-csp/#can-compile-strings:
// 'trusted-types-eval' only applies if Trusted Types are enforced ("does sink
// type require trusted types?" with includeReportOnly = false). Here they are
// not, so string compilation must be reported against script-src (but not
// blocked, since the policy is report-only).

// Install a pass-through default policy so that the (report-only) Trusted
// Types check does not report, and only the CSP script-src check is exercised.
trustedTypes.createPolicy('default', {createScript: s => s});

const kReportOnlyPolicy =
    'script-src \'self\' \'trusted-types-eval\'; require-trusted-types-for \'script\'';

function waitForViolation() {
  return new Promise(resolve => {
    window.addEventListener('securitypolicyviolation', resolve, {once: true});
  });
}

function checkViolation(e) {
  assert_equals(e.effectiveDirective, 'script-src');
  assert_equals(e.blockedURI, 'eval');
  assert_equals(e.originalPolicy, kReportOnlyPolicy);
  assert_equals(e.disposition, 'report');
}

var evalScriptRan = false;
var indirectEvalScriptRan = false;
var functionScriptRan = false;
var setTimeoutScriptRan = false;

promise_test(async t => {
  const violation = waitForViolation();
  eval('evalScriptRan = true;');
  assert_true(evalScriptRan);
  checkViolation(await violation);
}, 'Direct `eval` is allowed but reported.');

promise_test(async t => {
  const violation = waitForViolation();
  eval?.('indirectEvalScriptRan = true;');
  assert_true(indirectEvalScriptRan);
  checkViolation(await violation);
}, 'Indirect `eval` is allowed but reported.');

promise_test(async t => {
  const violation = waitForViolation();
  new Function('functionScriptRan = true;')();
  assert_true(functionScriptRan);
  checkViolation(await violation);
}, '`new Function` is allowed but reported.');

promise_test(async t => {
  const violation = waitForViolation();
  setTimeout('setTimeoutScriptRan = true;', 0);
  checkViolation(await violation);
  await new Promise(resolve => step_timeout(resolve, 0));
  assert_true(setTimeoutScriptRan);
}, '`setTimeout` is allowed but reported.');
