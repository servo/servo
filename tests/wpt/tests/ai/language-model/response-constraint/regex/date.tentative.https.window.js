// META: title=Language Model Response Regex - Date
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const regex = /^\d{4}-\d{2}-\d{2}$/;
  const response = await session.prompt(
      'Extract the date as YYYY-MM-DD from "Last edited on June 15 2026."',
      {responseConstraint: regex});
  assert_true(typeof response === 'string');
  assert_true(regex.test(response),
              `Response "${response}" should match regex ${regex}`);
}, 'Prompt should work with a date regex constraint.');
