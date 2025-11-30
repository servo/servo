// META: title=Language Model Clone
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  await ensureLanguageModel();

  // Start a new session and test it.
  const session = await createLanguageModel();
  const result = await session.prompt(kTestPrompt);
  assert_equals(typeof result, 'string');

  // Clone a session and test it.
  const cloned_session = await session.clone();
  assert_equals(
    cloned_session.inputQuota, session.inputQuota,
    'cloned session should have the same inputQuota as the original session.'
  );
  assert_equals(
    cloned_session.inputUsage, session.inputUsage,
    'cloned session should have the same inputUsage as the original session.'
  );
  assert_equals(
    cloned_session.topK, session.topK,
    'cloned session should have the same topK as the original session.'
  );
  assert_equals(
    cloned_session.temperature, session.temperature,
    'cloned session should have the same temperature as the original session.'
  );

  const clone_result = await cloned_session.prompt(kTestPrompt);
  assert_equals(typeof clone_result, 'string');
  assert_greater_than(
      cloned_session.inputUsage, session.inputUsage,
      'cloned session should have increased inputUsage after prompting.');
});
