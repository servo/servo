// META: title=Translate from English to Japanese
// META: global=window
// META: timeout=long
// META: script=../resources/util.js
// META: script=../resources/language_codes.js
// META: script=/resources/testdriver.js
//
// Setting `timeout=long` as this test may require downloading the translation
// library and the language models.

'use strict';

async function createTranslator(options) {
  return await test_driver.bless('Create translator', async () => {
    return await ai.translator.create(options);
  });
}

promise_test(async t => {
  const languagePair = {sourceLanguage: 'en', targetLanguage: 'ja'};

  // Creating the translator without user activation rejects with
  // NotAllowedError.
  const createPromise = ai.translator.create(languagePair);
  await promise_rejects_dom(t, 'NotAllowedError', createPromise);

  // Creating the translator with user activation succeeds.
  await createTranslator(languagePair);

  // TODO(crbug.com/390459310): Replace with availability.
  //
  // Creating it should have switched it to readily.
  const capabilities = await ai.translator.capabilities();
  const {sourceLanguage, targetLanguage} = languagePair;
  assert_equals(
      capabilities.languagePairAvailable(sourceLanguage, targetLanguage),
      'readily');

  // Now that it is readily, we should no longer need user activation.
  await ai.translator.create(languagePair);
}, 'AITranslator.create() requires user activation when availability is "after-download".');

promise_test(async t => {
  const translator =
      await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja'});
  assert_equals(await translator.translate('hello'), 'こんにちは');
}, 'Simple AITranslator.translate() call');

promise_test(async () => {
  const translator =
      await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja'});
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
      await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja'});
  assert_equals(translator.sourceLanguage, 'en');
  assert_equals(translator.targetLanguage, 'ja');
}, 'AITranslator: sourceLanguage and targetLanguage are equal to their respective option passed in to AITranslatorFactory.create.')

promise_test(async (t) => {
  const translator =
      await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja'});
  translator.destroy();
  await promise_rejects_dom(
      t, 'InvalidStateError', translator.translate('hello'));
}, 'AITranslator.translate() fails after destroyed');

promise_test(async t => {
  const controller = new AbortController();
  controller.abort();

  const createPromise = createTranslator(
      {signal: controller.signal, sourceLanguage: 'en', targetLanguage: 'ja'});

  await promise_rejects_dom(t, 'AbortError', createPromise);
}, 'AITranslatorFactory.create() call with an aborted signal.');

promise_test(async t => {
  await testAbortPromise(t, signal => {
    return createTranslator(
        {signal, sourceLanguage: 'en', targetLanguage: 'ja'});
  });
}, 'Aborting AITranslatorFactory.create().');

promise_test(async t => {
  const controller = new AbortController();
  controller.abort();

  const translator =
      await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja'});
  const translatePromise =
      translator.translate('hello', {signal: controller.signal});

  await promise_rejects_dom(t, 'AbortError', translatePromise);
}, 'AITranslator.translate() call with an aborted signal.');

promise_test(async t => {
  const translator =
      await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja'});
  await testAbortPromise(t, signal => {
    return translator.translate('hello', {signal});
  });
}, 'Aborting AITranslator.translate().');

promise_test(async t => {
  let monitorCalled = false;
  const progressEvents = [];
  function monitor(m) {
    monitorCalled = true;

    m.addEventListener('downloadprogress', e => {
      progressEvents.push(e);
    });
  }

  await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja', monitor});

  // Monitor callback must be called.
  assert_true(monitorCalled);

  // Must have at least 2 progress events, one for 0 and one for 1.
  assert_greater_than_equal(progressEvents.length, 2);
  assert_equals(progressEvents.at(0).loaded, 0);
  assert_equals(progressEvents.at(1).loaded, 1);

  // All progress events must have a total of 1.
  for (const progressEvent of progressEvents) {
    assert_equals(progressEvent.total, 1);
  }
}, 'AITranslatorFactory.create() monitor option is called correctly.');

promise_test(async t => {
  const translator =
      await ai.translator.create({sourceLanguage: 'en', targetLanguage: 'ja'});

  // Strings containing only white space are not translatable.
  const nonTranslatableStrings = ['', ' ', '     ', ' \r\n\t\f'];

  // Strings containing only control characters are not translatable.
  for (let c = 0; c < 0x1F; c++) {
    nonTranslatableStrings.push(String.fromCharCode(c));
  }

  const translatedNonTranslatableString = await Promise.all(
      nonTranslatableStrings.map(str => translator.translate(str)));

  // Non translatable strings should be echoed back
  assert_array_equals(translatedNonTranslatableString, nonTranslatableStrings);

  // Adding translatable text makes it translatable.
  const translatableStrings =
      nonTranslatableStrings.map(str => `Hello ${str} world`);

  const translatedTranslatableString = await Promise.all(
      translatableStrings.map(str => translator.translate(str)));

  // All the strings should have been translated in some way.
  for (let i = 0; i < translatableStrings.length; i++) {
    assert_not_equals(translatedTranslatableString[i], translatableStrings[i]);
  }
}, 'AITranslator.translate() echos non-translatable content');
