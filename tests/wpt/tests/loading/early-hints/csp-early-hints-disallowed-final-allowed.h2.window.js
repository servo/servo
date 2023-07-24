// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

test(() => {
    const early_hints_policy = "disallowed";
    const final_policy = "allowed";
    navigateToContentSecurityPolicyBasicTest(early_hints_policy, final_policy);
});
