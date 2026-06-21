// META: title=Language Model Response Regex - Exact Length
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const regex = /^.{100}$/;
  const response = await session.prompt(
      'Write a short story that is exactly 100 characters long.',
      {responseConstraint: regex});
  assert_true(typeof response === 'string');
  assert_true(regex.test(response),
              `Response "${response}" should match regex ${regex}`);
  assert_equals(response.length, 100, 'Response length should be exactly 100');
}, 'Prompt should work with an exact length regex constraint.');
