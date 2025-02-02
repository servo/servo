// META: title=Translate from English to Japanese
// META: global=window,worker
// META: timeout=long
// META: script=../resources/util.js
//
// Setting `timeout=long` as this test may require downloading the translation
// library and the language models.

'use strict';

promise_test(async t => {
  const translatorFactory = ai.translator;
  assert_not_equals(translatorFactory, null);
  const translator = await translatorFactory.create(
      {sourceLanguage: 'en', targetLanguage: 'ja'});
  assert_equals(await translator.translate('hello'), 'こんにちは');
}, 'Simple AITranslator.translate() call');

promise_test(async () => {
  const translator =
      await ai.translator.create({sourceLanguage: 'en', targetLanguage: 'ja'});
  const streamingResponse = translator.translateStreaming('hello');
  assert_equals(
      Object.prototype.toString.call(streamingResponse),
      '[object ReadableStream]');
  let result = '';
  for await (const chunk of streamingResponse) {
    result += chunk;
  }
  assert_equals(await translator.translate('hello'), 'こんにちは');
}, 'Simple AITranslator.translateStreaming() call');

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

promise_test(async t => {
  const controller = new AbortController();
  controller.abort();

  const createPromise = ai.translator.create(
      {signal: controller.signal, sourceLanguage: 'en', targetLanguage: 'ja'});

  await promise_rejects_dom(t, 'AbortError', createPromise);
}, 'AITranslatorFactory.create() call with an aborted signal.');

promise_test(async t => {
  await testAbortPromise(t, signal => {
    return ai.translator.create(
        {signal, sourceLanguage: 'en', targetLanguage: 'ja'});
  });
}, 'Aborting AITranslatorFactory.create().');

promise_test(async t => {
  const controller = new AbortController();
  controller.abort();

  const translator =
      await ai.translator.create({sourceLanguage: 'en', targetLanguage: 'ja'});
  const translatePromise =
      translator.translate('hello', {signal: controller.signal});

  await promise_rejects_dom(t, 'AbortError', translatePromise);
}, 'AITranslator.translate() call with an aborted signal.');

promise_test(async t => {
  const translator =
      await ai.translator.create({sourceLanguage: 'en', targetLanguage: 'ja'});
  await testAbortPromise(t, signal => {
    return translator.translate('hello', {signal});
  });
}, 'Aborting AITranslator.translate().');
