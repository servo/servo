// META: title=Language Model Prompt Multimodal Mix Post-Abort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const options = {
    expectedInputs: [{type: 'audio'}, {type: 'image'}]
  };
  await ensureLanguageModel(options);
  const audioBlob = await (await fetch(kValidAudioPath)).blob();
  const imageBlob = await (await fetch(kValidImagePath)).blob();
  const session = await createLanguageModel(options);

  const mixedMessage = [{
    role: 'user',
    content: [
      {type: 'text', value: 'compare this image and audio'},
      {type: 'audio', value: audioBlob},
      {type: 'image', value: imageBlob}
    ]
  }];

  const controller = new AbortController();
  const promise = session.prompt(mixedMessage, { signal: controller.signal });
  controller.abort();
  await promise_rejects_dom(t, 'AbortError', promise);

  // Prompt again on the same session to ensure it is still usable.
  const result = await session.prompt(kTestPrompt);
  assert_greater_than(result.length, 0);
}, "Multimodal mix prompt after aborting a previous multimodal mix prompt.");
