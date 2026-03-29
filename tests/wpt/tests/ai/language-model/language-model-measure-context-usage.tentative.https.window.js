// META: title=Language Model Measure Input Usage
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
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
