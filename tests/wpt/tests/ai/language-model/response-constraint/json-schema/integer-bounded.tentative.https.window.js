// META: title=Language Model Response JSON Schema - Bounded Integer
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: script=util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const response = await session.prompt(
      'Derive a rating between -10 and 10 from "Absolutely the best meal ever!"',
      {responseConstraint: {type: 'integer', minimum: -10, maximum: 10}});
  const jsonResponse = parse_json_response(response);
  assert_true(Number.isInteger(jsonResponse), 'Response should be an integer');
  assert_greater_than_equal(jsonResponse, -10, 'Response should be >= -10');
  assert_less_than_equal(jsonResponse, 10, 'Response should be <= 10');
}, 'Prompt should work with a bounded integer json schema constraint.');
