// META: title=Language Model Response JSON Schema - String
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
      'Extract a short quote from "Absolutely the best meal ever!"',
      {responseConstraint: {type: 'string'}});
  const jsonResponse = parse_json_response(response);
  assert_equals(typeof jsonResponse, 'string', 'Response should be a string');
}, 'Prompt should work with a string json schema constraint.');
