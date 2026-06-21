// META: title=Language Model Response JSON Schema - Null
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
      await session.prompt('Return null', {responseConstraint: {type: 'null'}});
  const jsonResponse = parse_json_response(response);
  assert_equals(jsonResponse, null, 'Response should be null');
}, 'Prompt should work with a null json schema constraint.');
