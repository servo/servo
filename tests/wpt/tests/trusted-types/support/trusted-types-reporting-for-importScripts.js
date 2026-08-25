const testSetupPolicy = trustedTypes.createPolicy("p", { createScriptURL: s => s });

importScripts(testSetupPolicy.createScriptURL("/resources/testharness.js"));
importScripts(testSetupPolicy.createScriptURL("helper.sub.js"));
importScripts(testSetupPolicy.createScriptURL("csp-violations.js"));

const policy = trustedTypes.createPolicy("dummy", { createScriptURL: x => x });
const input = `./namespaces.js?${'A'.repeat(100)}`;

promise_test(async t => {
  await no_trusted_type_violation_for(_ =>
    importScripts(policy.createScriptURL(input))
  );
}, "No violation reported for importScripts with TrustedScriptURL.");

promise_test(async t => {
  let violation = await trusted_type_violation_for(TypeError, _ =>
    importScripts(input)
  );
  assert_equals(violation.blockedURI, "trusted-types-sink");
  assert_equals(violation.sample, `WorkerGlobalScope importScripts|${clipSampleIfNeeded(input)}`);
}, "Violation report for importScripts with plain string.");

done();
