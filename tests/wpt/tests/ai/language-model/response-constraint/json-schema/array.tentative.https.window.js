// META: title=Language Model Response JSON Schema - Array
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
      'Extract the names as a JSON array from "I met John, Jane, and Jessie."',
      {responseConstraint: {type: 'array'}});
  const jsonResponse = parse_json_response(response);
  assert_true(Array.isArray(jsonResponse), 'Response should be an array');
}, 'Prompt should work with an array json schema constraint.');
