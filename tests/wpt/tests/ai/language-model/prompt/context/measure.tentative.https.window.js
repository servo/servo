// META: title=Language Model Measure Input Usage
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const result = await session.measureContextUsage('This is a prompt.');
  assert_equals(typeof result, 'number');
  assert_greater_than(result, 0);
}, 'measureContextUsage returns a number greater than zero for text');

promise_test(async t => {
  const prompts = [
    {role: 'system', content: 'foo'},
    {role: 'user', content: 'bar'},
    {role: 'assistant', content: 'baz'},
  ];
  await ensureLanguageModel();
  const session = await createLanguageModel({initialPrompts: prompts});
  const result = await session.measureContextUsage(prompts);
  assert_equals(typeof result, 'number');
  assert_greater_than(result, 0);
}, 'measure message sequences of various roles, even after adding prompts');

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const baseUsage = await session.measureContextUsage(kTestPrompt);
  const schema = {
    type: 'object',
    properties: {
      rating: {type: 'number'},
    },
  };

  const omittedSchemaUsage = await session.measureContextUsage(kTestPrompt, {
    responseConstraint: schema,
    omitResponseConstraintInput: true,
  });
  assert_equals(omittedSchemaUsage, baseUsage);

  const omittedRegexUsage = await session.measureContextUsage(kTestPrompt, {
    responseConstraint: /^[a-z]+$/,
    omitResponseConstraintInput: true,
  });
  assert_equals(omittedRegexUsage, baseUsage);
}, 'measureContextUsage omits responseConstraint input when omitResponseConstraintInput is true');

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const baseUsage = await session.measureContextUsage(kTestPrompt);
  const schema = {
    type: 'object',
    properties: {
      rating: {type: 'number'},
    },
  };

  const constrainedUsage = await session.measureContextUsage(
      kTestPrompt, {responseConstraint: schema});
  assert_implements_optional(
      constrainedUsage > baseUsage,
      'Implementation does not insert input for JSON schema responseConstraint');
}, 'measureContextUsage increases usage when responseConstraint input is inserted');

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const circularSchema = {};
  circularSchema.self = circularSchema;
  await promise_rejects_dom(
      t, 'NotSupportedError',
      session.measureContextUsage(kTestPrompt,
                                  {responseConstraint: circularSchema}));
}, 'measureContextUsage rejects response schemas with circular references');
