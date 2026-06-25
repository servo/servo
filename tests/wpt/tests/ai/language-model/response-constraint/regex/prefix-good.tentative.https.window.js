// META: title=Language Model Response Regex - Matching Prefix
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const goodPrefix = 'Greetings';
  const regex = /^Greetings and salutations.*/;
  const response = await session.prompt(
      [
        {role: 'user', content: 'hello'},
        {role: 'assistant', content: goodPrefix, prefix: true}
      ],
      {responseConstraint: regex});
  assert_true(typeof response === 'string');
  assert_true(
      regex.test(goodPrefix + response),
      `Response "${goodPrefix + response}" should match regex ${regex}`);
}, 'Prompt should work with a valid regex constraint and matching prefix.');
