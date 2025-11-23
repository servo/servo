// META: title=Language Model Response JSON Schema
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

const kValidResponseSchema = {
  type: 'object',
  required: ['Rating'],
  additionalProperties: false,
  properties: {
    Rating: {
      type: 'number',
      minimum: 0,
      maximum: 5,
    },
  },
};

function testResponseJsonSchema(response, t) {
  let jsonResponse;
  try {
    jsonResponse = JSON.parse(response);
  } catch (e) {
    assert_unreached(
        `Response is not valid JSON: "${response}". Error: ${e.message}`);
    return;
  }
  assert_equals(typeof jsonResponse, 'object', 'Response should be an object');
  assert_own_property(
      jsonResponse, 'Rating', 'JSON response should have a "Rating" property.');
  assert_equals(
      typeof jsonResponse.Rating, 'number', 'Rating should be a number');
  assert_greater_than_equal(jsonResponse.Rating, 0, 'Rating should be >= 0');
  assert_less_than_equal(jsonResponse.Rating, 5, 'Rating should be <= 5');
}

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
      'Response json schema is invalid - it should be an object that can be stringified into a JSON string.');
}, 'Prompt API should fail if an invalid response json schema is provided');

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const response =
      await session.prompt('hello', {responseConstraint: kValidResponseSchema});
  testResponseJsonSchema(response, t);
}, 'Prompt API should work when a valid response json schema is provided.');

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const assistantPrefix = '{ "Rating": ';
  const assistantResponse = await session.prompt(
      [
        {role: 'user', content: 'hello'},
        {role: 'assistant', content: assistantPrefix, prefix: true}
      ],
      {responseConstraint: kValidResponseSchema});
  const response = assistantPrefix + assistantResponse;
  testResponseJsonSchema(response, t);
}, 'Prompt API should work when a valid response json schema and model prefix is provided.');

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const response = await session.prompt('hello', {
    responseConstraint: kValidResponseSchema,
    omitResponseConstraintInput: true
  });
  testResponseJsonSchema(response, t);
}, 'Prompt API should omit response schema from input.');

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const promptPromise =
      session.prompt(kTestPrompt, {responseConstraint: /hello/});
  const result = await promptPromise;
  assert_true(typeof result === 'string');
}, 'Prompt API should work when a valid regex constraint is provided.');
