importScripts("/resources/testharness.js");
importScripts("csp-violations.js");

// For CSP applying to this file, please refer to
// should-trusted-type-policy-creation-be-blocked-by-csp-004-worker-tt-policy-name.js.headers
const tt_directive = `tt-policy-name-1 tt-policy-name-2 tt-policy-name-3`;

promise_test(async () => {
  await no_trusted_type_violation_for(_ => trustedTypes.createPolicy("tt-policy-name-1"));
  await no_trusted_type_violation_for(_ => trustedTypes.createPolicy("tt-policy-name-2"));
  await no_trusted_type_violation_for(_ => trustedTypes.createPolicy("tt-policy-name-3"));
} , `No violation/exception for allowed policy names (${tt_directive}).`);

promise_test(async () => {
  await trusted_type_violation_for(TypeError, _ => trustedTypes.createPolicy("tt-policy-name-1"));
  await trusted_type_violation_for(TypeError, _ => trustedTypes.createPolicy("tt-policy-name-2"));
  await trusted_type_violation_for(TypeError, _ => trustedTypes.createPolicy("tt-policy-name-3"));
}, `Violation and exception for duplicate policy names (${tt_directive}).`);

promise_test(async () => {
  await trusted_type_violation_for(TypeError, _ => trustedTypes.createPolicy("tt-policy-name-4"));
}, `Violation and exception for forbidden policy name (${tt_directive}).`);

done();
