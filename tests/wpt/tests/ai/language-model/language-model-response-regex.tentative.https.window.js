// META: title=Language Model Response Regex
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const response =
      await session.prompt(kTestPrompt, {responseConstraint: /hello/});
  assert_true(typeof response === 'string');
}, 'Prompt should work with a valid regex constraint.');

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const goodPrefix = 'Greetings';
  const response = await session.prompt(
      [
        {role: 'user', content: 'hello'},
        {role: 'assistant', content: goodPrefix, prefix: true}
      ],
      {responseConstraint: /^Greetings and salutations.*/});
  assert_true(typeof response === 'string');
}, 'Prompt should work with a valid regex constraint and matching prefix.');

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
          {responseConstraint: /^Greetings and salutations.*/}),
      'The request is invalid - the input or options could not be processed.');
}, 'Prompt should reject if the prefix deviates from the regex constraint.');
