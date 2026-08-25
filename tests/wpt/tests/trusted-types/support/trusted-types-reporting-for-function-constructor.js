const policy = trustedTypes.createPolicy("dummy", { createScript: x => x });

const AsyncFunction = async function() {}.constructor;
const GeneratorFunction = function*() {}.constructor;
const AsyncGeneratorFunction = async function*() {}.constructor;

const input = `return${';'.repeat(100)}`;
[Function, AsyncFunction, GeneratorFunction, AsyncGeneratorFunction].forEach(functionConstructor => {
  promise_test(async t => {
    await no_trusted_type_violation_for(_ =>
      new functionConstructor(policy.createScript(input))
    );
  }, `No violation reported for ${functionConstructor.name} with TrustedScript.`);

  promise_test(async t => {
    await no_trusted_type_violation_for(_ =>
      new functionConstructor(policy.createScript('a'), policy.createScript(input))
    );
  }, `No violation reported for ${functionConstructor.name} with multiple TrustedScript args.`);

  promise_test(async t => {
    let violation = await trusted_type_violation_for(EvalError, _ =>
      new functionConstructor(input)
    );
    assert_true(violation.originalPolicy.includes("require-trusted-types-for 'script'"));
    assert_equals(violation.blockedURI, "trusted-types-sink");
    assert_equals(violation.sample, `Function|${clipSampleIfNeeded(`(\n) {\n${input}\n}`)}`);
  }, `Violation report for ${functionConstructor.name} with plain string.`);
});
