// META: title=Language Model Response JSON Schema - Integer
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
      {responseConstraint: {type: 'integer'}});
  const jsonResponse = parse_json_response(response);
  assert_true(Number.isInteger(jsonResponse), 'Response should be an integer');
}, 'Prompt should work with an integer json schema constraint.');
