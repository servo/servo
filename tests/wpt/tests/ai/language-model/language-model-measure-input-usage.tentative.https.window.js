// META: title=Language Model Measure Input Usage
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();

  // Start a new session.
  const session = await createLanguageModel();

  // Test the measureInputUsage() API.
  let result = await session.measureInputUsage("This is a prompt.");
  assert_true(
    typeof result === "number" && result > 0,
    "The counting result should be a positive number."
  );
});
