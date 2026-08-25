// META: title=Language Model Response Regex - Time
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const regex = /^\d{2}:\d{2}(:\d{2})?$/;
  const response = await session.prompt(
      'Extract the 24 hour time as HH:MM from "The time is eleven thirty am"',
      {responseConstraint: regex});
  assert_true(typeof response === 'string');
  assert_true(regex.test(response),
              `Response "${response}" should match regex ${regex}`);
}, 'Prompt should work with a time regex constraint.');
