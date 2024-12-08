// META: title=translator.create without options do not crash
// META: global=window,worker
// META: timeout=long
//
// Setting `timeout=long` as this test may require downloading the translation
// library and the language models.

'use strict';

promise_test(async t => {
  const translatorFactory = ai.translator;
  assert_not_equals(translatorFactory, null);
  await promise_rejects_dom(t, 'InvalidStateError',translatorFactory.create(/*empty options*/),
    'No options are provided.');
});
