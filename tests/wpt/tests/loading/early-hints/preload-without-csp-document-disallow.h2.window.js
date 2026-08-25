// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

const early_hints_policy = "absent";
fetch_tests_from_window(navigateToContentSecurityPolicyDocumentDisallowTest(early_hints_policy));
