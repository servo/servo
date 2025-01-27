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

promise_test(async t => {
  const translator =
      await ai.translator.create({sourceLanguage: 'en', targetLanguage: 'ja'});
  assert_equals(translator.sourceLanguage, 'en');
  assert_equals(translator.targetLanguage, 'ja');
}, 'AITranslator: sourceLanguage and targetLanguage are equal to their respective option passed in to AITranslatorFactory.create.')

promise_test(async (t) => {
  const translator =
      await ai.translator.create({sourceLanguage: 'en', targetLanguage: 'ja'});
  translator.destroy();
  await promise_rejects_dom(
      t, 'InvalidStateError', translator.translate('hello'));
}, 'AITranslator.translate() fails after destroyed');
