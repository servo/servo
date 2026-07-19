// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

const early_hints_policy = "require-corp";
const final_policy = "unsafe-none";
fetch_tests_from_window(navigateToCrossOriginEmbedderPolicyMismatchTest(early_hints_policy,
    final_policy));
