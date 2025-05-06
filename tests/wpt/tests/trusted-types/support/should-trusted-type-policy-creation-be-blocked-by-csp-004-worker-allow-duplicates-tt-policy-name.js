importScripts("/resources/testharness.js");
importScripts("csp-violations.js");

// For CSP applying to this file, please refer to
// should-trusted-type-policy-creation-be-blocked-by-csp-004-worker-allow-duplicates-tt-policy-name.js.headers
const tt_directive = `'allow-duplicates' tt-policy-name`;

promise_test(async () => {
  await no_trusted_type_violation_for(_ => trustedTypes.createPolicy("tt-policy-name"));
} , `No violation/exception for allowed policy name (${tt_directive}).`);

promise_test(async () => {
  await no_trusted_type_violation_for(_ => trustedTypes.createPolicy("tt-policy-name"));
} , `No violation/exception for duplicate policy name (${tt_directive}).`);

promise_test(async () => {
  await trusted_type_violation_for(TypeError, _ => trustedTypes.createPolicy("duplicate"));
}, `Violation and exception for forbidden policy name 'duplicate' ${tt_directive}.`);

done();
