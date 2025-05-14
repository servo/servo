// META: title=translator.create without options do not crash
// META: global=window
// META: timeout=long
//
// Setting `timeout=long` as this test may require downloading the translation
// library and the language models.

'use strict';

promise_test(async t => {
  await promise_rejects_js(t, TypeError, Translator.create(/*empty options*/));
}, 'Translator.create rejects with TypeError if no options are passed.');

promise_test(async t => {
  await promise_rejects_js(
      t, TypeError, Translator.create({sourceLanguage: 'en'}));
}, 'Translator.create rejects with TypeError targetLanguage is not provided.');

promise_test(async t => {
  await promise_rejects_js(
      t, TypeError, Translator.create({targetLanguage: 'en'}));
}, 'Translator.create rejects with TypeError sourceLanguage is not provided.');
