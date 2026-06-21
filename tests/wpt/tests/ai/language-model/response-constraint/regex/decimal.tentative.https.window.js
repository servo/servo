// META: title=Language Model Response Regex - Decimal
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const regex = /^-?\d(\.\d+)?$/;
  const response = await session.prompt(
      'Derive a rating between -1.0 and 1.0 from "Absolutely the best meal ever!"',
      {responseConstraint: regex});
  assert_true(typeof response === 'string');
  assert_true(regex.test(response),
              `Response "${response}" should match regex ${regex}`);
  const num = parseFloat(response);
  assert_false(isNaN(num),
               `Response "${response}" should parse to a valid decimal`);
  assert_greater_than_equal(num, -1.0, 'Response should be >= -1.0');
  assert_less_than_equal(num, 1.0, 'Response should be <= 1.0');
}, 'Prompt should work with a decimal regex constraint.');
