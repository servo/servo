// META: title=Translator Translate Post-Abort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: script=resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const translator = await createTranslator({ sourceLanguage: 'en', targetLanguage: 'ja' });
  const controller = new AbortController();
  const promise = translator.translate('hello', { signal: controller.signal });
  controller.abort();
  await promise_rejects_dom(t, 'AbortError', promise);

  // Translate again on the same session to ensure it is still usable.
  const result = await translator.translate('hello');
  assert_greater_than(result.length, 0);
}, "Translate after aborting a previous translate.");
