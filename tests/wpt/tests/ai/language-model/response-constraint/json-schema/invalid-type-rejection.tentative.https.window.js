// META: title=Language Model Response JSON Schema - Invalid Type Rejection
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  // The type does not conform to any valid JSON schema type.
  const invalidResponseJsonSchema = {'type': 'soup'};
  await promise_rejects_dom(
      t, 'NotSupportedError',
      session.prompt(
          kTestPrompt, {responseConstraint: invalidResponseJsonSchema}),
      'Response constraint is not a supported json schema.');
}, 'Prompt should reject response schemas with invalid types');
