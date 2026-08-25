// META: title=Language Model Prompt Multimodal Audio Dual Post-Abort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel(kAudioOptions);
  const audioBuffer = await (await fetch(kValidAudioPath)).arrayBuffer();
  const session = await createLanguageModel(kAudioOptions);

  const message = [{
    role: 'user',
    content: [
      {type: 'audio', value: audioBuffer},
      {type: 'audio', value: audioBuffer},
      {type: 'text', value: 'compare these audio files'}
    ]
  }];

  const controller = new AbortController();
  const promise = session.prompt(message, { signal: controller.signal });
  controller.abort();
  await promise_rejects_dom(t, 'AbortError', promise);

  // Prompt again on the same session to ensure it is still usable.
  const result = await session.prompt(kTestPrompt);
  assert_greater_than(result.length, 0);
}, "Multimodal prompt with dual audio inputs after aborting a previous request.");
