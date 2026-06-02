const policy = trustedTypes.createPolicy("dummy", { createScriptURL: x => x });
const input = `data:text/javascript,${';'.repeat(100)}`;

promise_test(async t => {
  await no_trusted_type_violation_for(_ =>
    new SharedWorker(policy.createScriptURL(input))
  );
}, "No violation reported for SharedWorker constructor with TrustedScriptURL.");

promise_test(async t => {
  let violation = await trusted_type_violation_for(TypeError, _ =>
    new SharedWorker(input)
  );
  assert_equals(violation.blockedURI, "trusted-types-sink");
  assert_equals(violation.sample, `SharedWorker constructor|${clipSampleIfNeeded(input)}`);
}, "Violation report for SharedWorker constructor with plain string.");
