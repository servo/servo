// META: title=Optional Translator tests
// META: global=window
// META: timeout=long
// META: script=../resources/util.js
// META: script=/resources/testdriver.js
// META: script=resources/util.js
//
// Setting `timeout=long` as this test may require downloading the translation
// library and the language models.

'use strict';

promise_test(async t => {
  const languagePair = {sourceLanguage: 'en', targetLanguage: 'ja'};

  // Creating the translator without user activation rejects with
  // NotAllowedError.
  const createPromise = Translator.create(languagePair);
  await promise_rejects_dom(t, 'NotAllowedError', createPromise);

  // Creating the translator with user activation succeeds.
  await createTranslator(languagePair);

  // Creating it should have switched it to available.
  const availability = await Translator.availability(languagePair);
  assert_equals(availability, 'available');

  // Now that it is available, we should no longer need user activation.
  await Translator.create(languagePair);
}, 'Translator.create() requires user activation when availability is "downloadable.');

promise_test(async t => {
  const translator =
      await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja'});
  assert_equals(await translator.translate('hello'), 'こんにちは');
}, 'Simple Translator.translate() call');

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
}, 'Simple Translator.translateStreaming() call');

promise_test(async t => {
  const translator =
      await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja'});
  assert_equals(translator.sourceLanguage, 'en');
  assert_equals(translator.targetLanguage, 'ja');
}, 'Translator: sourceLanguage and targetLanguage are equal to their respective option passed in to Translator.create.');

promise_test(async t => {
  const controller = new AbortController();
  controller.abort();

  const createPromise = createTranslator(
      {signal: controller.signal, sourceLanguage: 'en', targetLanguage: 'ja'});

  await promise_rejects_dom(t, 'AbortError', createPromise);
}, 'Translator.create() call with an aborted signal.');

promise_test(async t => {
  await testAbortPromise(t, signal => {
    return createTranslator(
        {signal, sourceLanguage: 'en', targetLanguage: 'ja'});
  });
}, 'Aborting Translator.create().');

promise_test(async t => {
  const controller = new AbortController();
  controller.abort();

  const translator =
      await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja'});
  const translatePromise =
      translator.translate('hello', {signal: controller.signal});

  await promise_rejects_dom(t, 'AbortError', translatePromise);
}, 'Translator.translate() call with an aborted signal.');

promise_test(async t => {
  const translator =
      await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja'});
  await testAbortPromise(t, signal => {
    return translator.translate('hello', {signal});
  });
}, 'Aborting Translator.translate().');

promise_test(async t => {
  const translator =
      await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja'});

  const text = 'this string is in English';
  const promises =
      [translator.translate(text), translator.measureInputUsage(text)];

  translator.destroy();

  promises.push(translator.translate(text), translator.measureInputUsage(text));

  for (const promise of promises) {
    await promise_rejects_dom(t, 'AbortError', promise);
  }
}, 'Calling Translator.destroy() aborts calls to translate and measureInputUsage.');

promise_test(async t => {
  const controller = new AbortController();
  const translator = await createTranslator(
      {sourceLanguage: 'en', targetLanguage: 'ja', signal: controller.signal});

  const text = 'this string is in English';
  const promises =
      [translator.translate(text), translator.measureInputUsage(text)];

  const error = new Error('The create abort signal was aborted.');
  controller.abort(error);

  promises.push(translator.translate(text), translator.measureInputUsage(text));

  for (const promise of promises) {
    await promise_rejects_exactly(t, error, promise);
  }
}, 'Translator.create()\'s abort signal destroys its Translator after creation.');


promise_test(async t => {
  await testMonitor(
      createTranslator, {sourceLanguage: 'en', targetLanguage: 'ja'});
}, 'Translator.create() notifies its monitor on downloadprogress');

promise_test(async t => {
  await testCreateMonitorWithAbort(
      t, createTranslator, {sourceLanguage: 'en', targetLanguage: 'ja'});
}, 'Progress events are not emitted after aborted.');

promise_test(async t => {
  const translator =
      await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja'});

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
}, 'Translator.translate() echoes non-translatable content');

promise_test(async t => {
  const translator =
      await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja'});

  const text = 'hello';
  const inputUsage = await translator.measureInputUsage(text);

  assert_greater_than_equal(translator.inputQuota, 0);
  assert_greater_than_equal(inputUsage, 0);

  if (inputUsage < translator.inputQuota) {
    assert_equals(await translator.translate(text), 'こんにちは');
  } else {
    await promise_rejects_dom(
        t, 'QuotaExceededError', translator.translate(text));
  }
}, 'Translator.measureInputUsage() and inputQuota basic usage.');

promise_test(async t => {
  const controller = new AbortController();
  controller.abort();

  const translator =
      await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja'});
  const measureInputUsagePromise =
      translator.measureInputUsage('hello', {signal: controller.signal});

  await promise_rejects_dom(t, 'AbortError', measureInputUsagePromise);
}, 'Translator.measureInputUsage() call with an aborted signal.');

promise_test(async t => {
  const translator =
      await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja'});
  await testAbortPromise(t, signal => {
    return translator.measureInputUsage('hello', {signal});
  });
}, 'Aborting Translator.measureInputUsage().');
