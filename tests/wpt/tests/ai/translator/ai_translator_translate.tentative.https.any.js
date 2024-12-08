// META: title=Translate from English to Japanese
// META: global=window,worker
// META: timeout=long
//
// Setting `timeout=long` as this test may require downloading the translation
// library and the language models.

'use strict';

promise_test(async t => {
  const translatorFactory = ai.translator;
  assert_not_equals(translatorFactory, null);
  const translator = await translatorFactory.create({
    sourceLanguage: "en",
    targetLanguage: "ja"
  });
  assert_equals(await translator.translate('hello'), 'こんにちは');
});
