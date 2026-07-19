// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

const early_hints_policy = "unsafe-none";
const final_policy = "require-corp";
fetch_tests_from_window(navigateToCrossOriginEmbedderPolicyMismatchTest(early_hints_policy,
    final_policy));
