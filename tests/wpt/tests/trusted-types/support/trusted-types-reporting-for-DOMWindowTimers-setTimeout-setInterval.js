const globalThisStr = getGlobalThisStr();
// https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#timer-initialisation-steps,
// step 9.6.1.1.
const expectedSinkPrefix = globalThisStr.includes("Window") ? "Window" : "WorkerGlobalScope";

const policy = trustedTypes.createPolicy("dummy", { createScript: x => x });
const input = ';'.repeat(100);

promise_test(async t => {
  await no_trusted_type_violation_for(_ =>
    setTimeout(policy.createScript(input))
  );
}, "No violation reported for setTimeout with TrustedScript.");

promise_test(async t => {
  await no_trusted_type_violation_for(_ =>
    setInterval(policy.createScript(input))
  );
}, "No violation reported for setInterval with TrustedScript.");

promise_test(async t => {
  let violation = await trusted_type_violation_for(TypeError, _ =>
    setTimeout(input)
  );
  assert_equals(violation.blockedURI, "trusted-types-sink");
  assert_equals(violation.sample, `${expectedSinkPrefix} setTimeout|${clipSampleIfNeeded(input)}`);
}, "Violation report for setTimeout with plain string.");

promise_test(async t => {
  let violation = await trusted_type_violation_for(TypeError, _ =>
    setInterval(input)
  );
  assert_equals(violation.blockedURI, "trusted-types-sink");
  assert_equals(violation.sample, `${expectedSinkPrefix} setInterval|${clipSampleIfNeeded(input)}`);
}, "Violation report for setInterval with plain string.");