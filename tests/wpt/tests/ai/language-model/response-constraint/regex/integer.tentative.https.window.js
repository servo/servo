// META: title=Language Model Response Regex - Integer
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const regex = /^-?\d$/;
  const response = await session.prompt(
      'Derive a rating between -9 and 9 from "Absolutely the best meal ever!"',
      {responseConstraint: regex});
  assert_true(typeof response === 'string');
  assert_true(regex.test(response),
              `Response "${response}" should match regex ${regex}`);
  const num = parseInt(response, 10);
  assert_false(isNaN(num),
               `Response "${response}" should parse to a valid integer`);
  assert_greater_than_equal(num, -9, 'Response should be >= -9');
  assert_less_than_equal(num, 9, 'Response should be <= 9');
}, 'Prompt should work with an integer regex constraint.');
