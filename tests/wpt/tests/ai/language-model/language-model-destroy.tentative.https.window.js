// META: title=Language Model Destroy
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();

  // Start a new session.
  const session = await createLanguageModel();

  // Calling `session.destroy()` immediately after `session.prompt()` will
  // trigger the "The model execution session has been destroyed." exception.
  let result = session.prompt(kTestPrompt);
  session.destroy();
  await promise_rejects_dom(
    t, "InvalidStateError", result,
    "The model execution session has been destroyed."
  );

  // Calling `session.prompt()` after `session.destroy()` will trigger the
  // "The model execution session has been destroyed." exception.
  await promise_rejects_dom(
    t, "InvalidStateError", session.prompt(kTestPrompt),
    "The model execution session has been destroyed."
  );

  // After destroying the session, the properties should be still accessible.
  assert_equals(
      typeof session.contextWindow, 'number',
      'contextWindow must be accessible.');
  assert_equals(
      typeof session.contextUsage, 'number',
      'contextUsage must be accessible.');
  assert_equals(
      typeof session.temperature, 'undefined', 'temperature is deprecated.');
  assert_equals(typeof session.topK, 'undefined', 'topK is deprecated.');
});
