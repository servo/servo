importScripts("/resources/testharness.js");
importScripts("csp-violations.js");

// For CSP applying to this file, please refer to
// should-trusted-type-policy-creation-be-blocked-by-csp-004-worker-multiple-violation.js.headers

promise_test(async () => {
  let {violations, exception} = await trusted_type_violations_and_exception_for(_ => trustedTypes.createPolicy("tt-policy-name"));

  // An exception is thrown for the violated enforced policies.
  assert_true(exception instanceof TypeError, "TypeError is thrown");

  // This violates other-policy-name and none directives.
  let sorted_violations = violations.map(v => {
    return { policy: v.originalPolicy, disposition: v.disposition};
  }).sort();
  assert_equals(sorted_violations.length, 4);
  assert_equals(sorted_violations[0].policy, "trusted-types other-policy-name");
  assert_equals(sorted_violations[0].disposition, "enforce");
  assert_equals(sorted_violations[1].policy, "trusted-types 'none'");
  assert_equals(sorted_violations[1].disposition, "enforce");
  assert_equals(sorted_violations[2].policy, "trusted-types other-policy-name");
  assert_equals(sorted_violations[2].disposition, "report");
  assert_equals(sorted_violations[3].policy, "trusted-types 'none'");
  assert_equals(sorted_violations[3].disposition, "report");
}, "Exception and violations for CSP with multiple enforce and report-only policies.");

done();
