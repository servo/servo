// META: title=Translate from English to Japanese
// META: global=window,worker
// META: timeout=long
//
// Setting `timeout=long` as this test may require downloading the translation
// library and the language models.

'use strict';

promise_test(async t => {
  const translator = await translation.createTranslator({
    sourceLanguage: 'en',
    targetLanguage: 'ja',
  });
  assert_equals(await translator.translate('hello'), 'こんにちは');
});
