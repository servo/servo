importScripts("/resources/testharness.js");
importScripts("csp-violations.js");

// For CSP applying to this file, please refer to
// should-trusted-type-policy-creation-be-blocked-by-csp-004-worker-none.js.headers
const tt_directive = `'none'`;

promise_test(async () => {
  await trusted_type_violation_for(TypeError, _ => trustedTypes.createPolicy("tt-policy-name"));
} , `Violation and exception for policy name "tt-policy-name" (${tt_directive}).`);

promise_test(async () => {
  await trusted_type_violation_for(TypeError, _ => trustedTypes.createPolicy("none"));
} , `Violation and exception for policy name "none" (${tt_directive}).`);

done();
