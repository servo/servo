// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

test(() => {
    const early_hints_policy = "allowed";
    navigateToContentSecurityPolicyDocumentDisallowTest(early_hints_policy);
});
