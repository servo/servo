// META: title=Translator Translate Concurrent (Optional)
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: script=resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const translator =
      await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja'});
  await Promise.all(
      [translator.translate(kTestPrompt), translator.translate(kTestPrompt)]);
}, 'Multiple Translator.translate() calls with identical inputs are resolved successfully');

promise_test(async () => {
  const translator =
      await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja'});
  await Promise.all(
      [translator.translate(kTestPrompt), translator.translate(kTestPrompt2)]);
}, 'Multiple Translator.translate() calls with divergent inputs are resolved successfully');
