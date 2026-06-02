const testSetupPolicy = trustedTypes.createPolicy("testSetupPolicy", {
  createScriptURL: s => s });

importScripts(testSetupPolicy.createScriptURL("/resources/testharness.js"));
importScripts(testSetupPolicy.createScriptURL("csp-violations.js"));

// For CSP applying to this file, please refer to
// should-sink-type-mismatch-violation-be-blocked-by-csp-002-worker-multiple-violations.js.headers

promise_test(async () => {
  let {violations, exception} = await trusted_type_violations_and_exception_for(_ => setTimeout(";;;;;"));

  // An exception is thrown for the violated enforced policies.
  assert_true(exception instanceof TypeError, "TypeError is thrown");

  // This violates all 'script' directives.
  let sorted_violations = violations.map(v => {
    return { policy: v.originalPolicy, disposition: v.disposition};
  }).sort();
  assert_equals(sorted_violations.length, 6);
  assert_equals(sorted_violations[0].policy, "require-trusted-types-for 'script'");
  assert_equals(sorted_violations[0].disposition, "enforce");
  assert_equals(sorted_violations[1].policy, "require-trusted-types-for 'script' 'invalid'");
  assert_equals(sorted_violations[1].disposition, "enforce");
  assert_equals(sorted_violations[2].policy, "require-trusted-types-for 'invalid' 'script'");
  assert_equals(sorted_violations[2].disposition, "enforce");
  assert_equals(sorted_violations[3].policy, "require-trusted-types-for 'script'");
  assert_equals(sorted_violations[3].disposition, "report");
  assert_equals(sorted_violations[4].policy, "require-trusted-types-for 'script' 'invalid'");
  assert_equals(sorted_violations[4].disposition, "report");
  assert_equals(sorted_violations[5].policy, "require-trusted-types-for 'invalid' 'script'");
  assert_equals(sorted_violations[5].disposition, "report");
}, "Checking reported violations for setTimeout(';;;;;') from DedicatedWorker");

done();
