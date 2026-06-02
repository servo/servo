importScripts("/resources/testharness.js");
importScripts("csp-violations.js");

// This test verifies that 'none' keyword is ignored if other tt-expression is
// present.

// For CSP applying to this file, please refer to
// should-trusted-type-policy-creation-be-blocked-by-csp-004-worker-tt-policy-name-none.js.headers
const tt_directive = `tt-policy-name 'none'`;

promise_test(async () => {
  await no_trusted_type_violation_for(_ => trustedTypes.createPolicy("tt-policy-name"));
} , `No violation/exception for allowed policy names (${tt_directive}).`);

promise_test(async () => {
  await trusted_type_violation_for(TypeError, _ => trustedTypes.createPolicy("other-policy-name"));
}, `Violation and exception for forbidden policy name (${tt_directive}).`);

done();
