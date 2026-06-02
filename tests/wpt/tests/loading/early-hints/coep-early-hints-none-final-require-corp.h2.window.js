// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

test(() => {
    const early_hints_policy = "unsafe-none";
    const final_policy = "require-corp";
    navigateToCrossOriginEmbedderPolicyMismatchTest(early_hints_policy,
        final_policy);
});
