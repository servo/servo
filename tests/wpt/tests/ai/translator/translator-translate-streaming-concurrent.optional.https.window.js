// META: title=Translator Translate Streaming Concurrent (Optional)
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: script=resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const translator =
      await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja'});
  const [result1, result2] = await Promise.all([
    readStream(translator.translateStreaming(kTestPrompt)),
    readStream(translator.translateStreaming(kTestPrompt)),
  ]);
  assert_greater_than(result1.length, 0);
  assert_greater_than(result2.length, 0);
}, 'Multiple Translator.translateStreaming() calls with identical inputs are resolved successfully');

promise_test(async () => {
  const translator =
      await createTranslator({sourceLanguage: 'en', targetLanguage: 'ja'});
  const [result1, result2] = await Promise.all([
    readStream(translator.translateStreaming(kTestPrompt)),
    readStream(translator.translateStreaming(kTestPrompt2)),
  ]);
  assert_greater_than(result1.length, 0);
  assert_greater_than(result2.length, 0);
}, 'Multiple Translator.translateStreaming() calls with divergent inputs are resolved successfully');
