const policy = trustedTypes.createPolicy("dummy", { createScriptURL: x => x });
const input = `data:text/javascript,${';'.repeat(100)}`;

promise_test(async t => {
  await no_trusted_type_violation_for(_ =>
    new Worker(policy.createScriptURL(input))
  );
}, "No violation reported for Worker constructor with TrustedScriptURL.");

promise_test(async t => {
  let violation = await trusted_type_violation_for(TypeError, _ =>
    new Worker(input)
  );
  assert_equals(violation.blockedURI, "trusted-types-sink");
  assert_equals(violation.sample, `Worker constructor|${clipSampleIfNeeded(input)}`);
}, "Violation report for Worker constructor with plain string.");
