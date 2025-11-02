// META: title=Language Model Response JSON Schema
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  // Circular reference is not valid.
  const invalidRepsonseJsonSchema = {};
  invalidRepsonseJsonSchema.self = invalidRepsonseJsonSchema;
  await promise_rejects_dom(t, 'NotSupportedError',
    session.prompt(kTestPrompt, { responseConstraint: invalidRepsonseJsonSchema }),
    'Response json schema is invalid - it should be an object that can be stringified into a JSON string.');
}, 'Prompt API should fail if an invalid response json schema is provided');

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const validRepsonseJsonSchema = {
    type: "object",
    required: ["Rating"],
    additionalProperties: false,
    properties: {
      Rating: {
        type: "number",
        minimum: 0,
        maximum: 5,
      },
    },
  };
  const promptPromise = session.prompt('hello', { responseConstraint : validRepsonseJsonSchema });
  // Both the prompt and schema should be present.
  assert_regexp_match(await promptPromise, /hello.*Rating/s);
}, 'Prompt API should work when a valid response json schema is provided.');

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const validRepsonseJsonSchema = {
    type: "object",
    required: ["Rating"],
    additionalProperties: false,
    properties: {
      Rating: {
        type: "number",
        minimum: 0,
        maximum: 5,
      },
    },
  };
  const promptPromise = session.prompt([
    {role: 'user', content: 'hello'},
    {role: 'assistant', content: 'prefix', prefix: true}
  ], { responseConstraint : validRepsonseJsonSchema });
  // Both the prompt and schema should be present, but prefix should be last.
  assert_regexp_match(await promptPromise, /hello.*Rating.*prefix/s);
}, 'Prompt API should work when a valid response json schema and model prefix is provided.');

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const validRepsonseJsonSchema = {
    type: "object",
    required: ["Rating"],
    additionalProperties: false,
    properties: {
      Rating: {
        type: "number",
        minimum: 0,
        maximum: 5,
      },
    },
  };
  const promptPromise = session.prompt('hello', {
    responseConstraint : validRepsonseJsonSchema,
    omitResponseConstraintInput : true
  });
  assert_regexp_match(await promptPromise, /hello$/);
}, 'Prompt API should omit response schema from input.');

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const promptPromise = session.prompt(kTestPrompt, { responseConstraint : /hello/ });
  const result = await promptPromise;
  assert_true(typeof result === "string");
}, 'Prompt API should work when a valid regex constraint is provided.');
