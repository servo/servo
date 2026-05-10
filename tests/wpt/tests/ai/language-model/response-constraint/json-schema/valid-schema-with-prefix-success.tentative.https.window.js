// META: title=Language Model Response JSON Schema - Valid Schema With Prefix Success
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const goodPrefix = '{ "Rating": ';
  const assistantResponse = await session.prompt(
      [
        {role: 'user', content: 'hello'},
        {role: 'assistant', content: goodPrefix, prefix: true}
      ],
      {responseConstraint: kValidResponseSchema});
  const response = goodPrefix + assistantResponse;
  testResponseJsonSchema(response, t);
}, 'Prompt should work when a valid response json schema and matching prefix is provided.');
