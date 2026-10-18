// META: title=A report-only policy without `trusted-types-eval` reports, but does not block, eval.

trustedTypes.createPolicy('default', {createScript: s => s});

const kReportOnlyPolicy = 'script-src \'self\'';

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
var functionScriptRan = false;

promise_test(async t => {
  const violation = waitForViolation();
  eval('evalScriptRan = true;');
  assert_true(evalScriptRan);
  checkViolation(await violation);
}, '`eval` is allowed but reported by a report-only policy without `trusted-types-eval`.');

promise_test(async t => {
  const violation = waitForViolation();
  new Function('functionScriptRan = true;')();
  assert_true(functionScriptRan);
  checkViolation(await violation);
}, '`new Function` is allowed but reported by a report-only policy without `trusted-types-eval`.');
