// META: title=Language Model Response JSON Schema - Valid Schema Success
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const response =
      await session.prompt('hello', {responseConstraint: kValidResponseSchema});
  testResponseJsonSchema(response, t);
}, 'Prompt should work when a valid response json schema is provided.');
