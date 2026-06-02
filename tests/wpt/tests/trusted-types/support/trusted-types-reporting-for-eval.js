const policy = trustedTypes.createPolicy("dummy", { createScript: x => x });

promise_test(async t => {
  let beacon = 'never_overwritten2';
  await no_trusted_type_violation_for(_ =>
    eval(policy.createScript('beacon="i ran"'))
  );
  assert_equals(beacon, 'i ran');
}, "No violation reported for eval with TrustedScript.");

promise_test(async t => {
  const input = 'beacon="should not run"';
  let beacon = 'never_overwritten';
  let violation = await trusted_type_violation_for(EvalError, _ =>
    eval(input)
  );
  assert_true(violation.originalPolicy.includes("require-trusted-types-for 'script'"));
  assert_equals(violation.blockedURI, "trusted-types-sink");
  assert_equals(violation.sample, `eval|${clipSampleIfNeeded(input)}`);
  assert_equals(beacon, 'never_overwritten');
}, "Violation report for eval with plain string.");