importScripts("/resources/testharness.js");
importScripts("csp-violations.js");

// For CSP applying to this file, please refer to
// should-trusted-type-policy-creation-be-blocked-by-csp-004-worker-wildcard.js.headers
const tt_directive = `tt-policy-name-1 * tt-policy-name-3`;

promise_test(async () => {
  await no_trusted_type_violation_for(TypeError, _ => trustedTypes.createPolicy("tt-policy-name-1"));
  await no_trusted_type_violation_for(TypeError, _ => trustedTypes.createPolicy("tt-policy-name-2"));
  await no_trusted_type_violation_for(TypeError, _ => trustedTypes.createPolicy("tt-policy-name-3"));
  await no_trusted_type_violation_for(TypeError, _ => trustedTypes.createPolicy("other-policy-name"));
} , `No violation and exception for allowed policy names (${tt_directive}).`);

done();
