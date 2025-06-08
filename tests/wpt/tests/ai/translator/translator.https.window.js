// META: title=Translator tests
// META: global=window
// META: timeout=long
// META: script=../resources/util.js
// META: script=/resources/testdriver.js
// META: script=resources/util.js
//
// Setting `timeout=long` as this test may require downloading the translation
// library and the language models.

promise_test(async t => {
  // Can pass in valid but unsupported languages since the create monitor error
  // should be thrown before language support is checked.
  await testCreateMonitorCallbackThrowsError(
      t, createTranslator, {sourceLanguage: 'und', targetLanguage: 'und'});
}, 'If monitor throws an error, LanguageDetector.create() rejects with that error');
