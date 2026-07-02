// META: title=Translator Translate Streaming Post-Abort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: script=resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const translator = await createTranslator({ sourceLanguage: 'en', targetLanguage: 'ja' });
  const controller = new AbortController();
  const stream = translator.translateStreaming('hello', { signal: controller.signal });
  controller.abort();
  await promise_rejects_dom(t, 'AbortError', stream.pipeTo(new WritableStream()));

  // Translate again on the same session to ensure it is still usable.
  const streamingResponse = translator.translateStreaming('hello');
  assert_true(streamingResponse instanceof ReadableStream);
  const result = (await Array.fromAsync(streamingResponse)).join('');
  assert_greater_than(result.length, 0, 'The result should not be empty.');
}, "Translate after aborting a previous translateStreaming.");
