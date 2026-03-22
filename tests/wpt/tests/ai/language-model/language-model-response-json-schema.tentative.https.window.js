// META: title=Language Model Response JSON Schema
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
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
      'Response constraint is not a supported json schema.');
}, 'Prompt should reject response schemas with circular references');

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

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const response =
      await session.prompt('hello', {responseConstraint: kValidResponseSchema});
  testResponseJsonSchema(response, t);
}, 'Prompt should work when a valid response json schema is provided.');

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

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const badPrefix = 'invalid';
  await promise_rejects_dom(
      t, 'NotSupportedError',
      session.prompt(
          [
            {role: 'user', content: 'hello'},
            {role: 'assistant', content: badPrefix, prefix: true}
          ],
          {responseConstraint: kValidResponseSchema}),
      'Response constraint is not a supported json schema.');
}, 'Prompt should reject if the prefix deviates from the json schema constraint.');

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const response = await session.prompt('hello', {
    responseConstraint: kValidResponseSchema,
    omitResponseConstraintInput: true
  });
  testResponseJsonSchema(response, t);
}, 'Prompt should omit response schema from input.');
