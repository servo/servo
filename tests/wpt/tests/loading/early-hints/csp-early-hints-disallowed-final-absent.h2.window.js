// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

const early_hints_policy = "disallowed";
const final_policy = "absent";
fetch_tests_from_window(navigateToContentSecurityPolicyBasicTest(early_hints_policy, final_policy));
