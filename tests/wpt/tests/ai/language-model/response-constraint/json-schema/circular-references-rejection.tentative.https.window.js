// META: title=Language Model Response JSON Schema - Circular References Rejection
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  // Circular reference is not valid.
  const invalidResponseJsonSchema = {};
  invalidResponseJsonSchema.self = invalidResponseJsonSchema;
  await promise_rejects_dom(
      t, 'NotSupportedError',
      session.prompt(
          kTestPrompt, {responseConstraint: invalidResponseJsonSchema}),
      'Response constraint is not a supported json schema.');
}, 'Prompt should reject response schemas with circular references');
