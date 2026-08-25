// META: title=Language Model Response JSON Schema - Object
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
      'Extract the key info as a JSON object from "John Doe is 30 years old"',
      {responseConstraint: {type: 'object'}});
  const jsonResponse = parse_json_response(response);
  assert_equals(typeof jsonResponse, 'object', 'Response should be an object');
  assert_not_equals(jsonResponse, null, 'Response should not be null');
}, 'Prompt should work with an object json schema constraint.');
