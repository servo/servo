// META: title=Language Model Response Regex - CSV Row
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const regex = /^([^,]+,)+[^,]+$/;
  const response = await session.prompt(
      'Extract the names as a comma-separated list from "I met John, Jane, and Jessie."',
      {responseConstraint: regex});
  assert_true(typeof response === 'string');
  assert_true(regex.test(response),
              `Response "${response}" should match regex ${regex}`);
}, 'Prompt should work with a CSV row regex constraint.');
