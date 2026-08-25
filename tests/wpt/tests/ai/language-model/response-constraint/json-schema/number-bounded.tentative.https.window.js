// META: title=Language Model Response JSON Schema - Bounded Number
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
      'Derive a rating between -1.0 and 1.0 from "Absolutely the best meal ever!"',
      {responseConstraint: {type: 'number', minimum: -1.0, maximum: 1.0}});
  const jsonResponse = parse_json_response(response);
  assert_equals(typeof jsonResponse, 'number', 'Response should be a number');
  assert_greater_than_equal(jsonResponse, -1.0, 'Response should be >= -1.0');
  assert_less_than_equal(jsonResponse, 1.0, 'Response should be <= 1.0');
}, 'Prompt should work with a bounded number json schema constraint.');
