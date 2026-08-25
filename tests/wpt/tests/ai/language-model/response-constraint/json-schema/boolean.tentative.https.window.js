// META: title=Language Model Response JSON Schema - Boolean
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: script=util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const response =
      await session.prompt('Do you want to play a game? Answer true or false.',
                           {responseConstraint: {type: 'boolean'}});
  const jsonResponse = parse_json_response(response);
  assert_equals(typeof jsonResponse, 'boolean', 'Response should be a boolean');
}, 'Prompt should work with a boolean json schema constraint.');
