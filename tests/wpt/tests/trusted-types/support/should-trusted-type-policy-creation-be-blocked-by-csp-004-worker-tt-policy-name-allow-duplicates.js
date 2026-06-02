importScripts("/resources/testharness.js");
importScripts("csp-violations.js");

// This test is similar to allow-duplicates-tt-policy-name.js but with a
// different ordering of tt-expressions.

// For CSP applying to this file, please refer to
// should-trusted-type-policy-creation-be-blocked-by-csp-004-worker-tt-policy-name-allow-duplicates.js.headers
const tt_directive = `tt-policy-name 'allow-duplicates'`;

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
