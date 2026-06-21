// META: title=Language Model Response JSON Schema - Number
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
      {responseConstraint: {type: 'number'}});
  const jsonResponse = parse_json_response(response);
  assert_equals(typeof jsonResponse, 'number', 'Response should be a number');
}, 'Prompt should work with a number json schema constraint.');
