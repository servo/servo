// META: title=Language Model Prompt Multimodal Audio
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async (t) => {
  await ensureLanguageModel();
  const blob = await (await fetch(kValidAudioPath)).blob();
  const session = await createLanguageModel();
  return promise_rejects_dom(
      t, 'NotSupportedError',
      session.prompt(messageWithContent(kImagePrompt, 'audio', blob)));
}, 'Prompt audio without `audio` expectedInput');

promise_test(async () => {
  const blob = await (await fetch(kValidAudioPath)).blob();
  const options = {
    expectedInputs: [{type: 'audio'}],
    initialPrompts: messageWithContent(kAudioPrompt, 'audio', blob)
  };
  await ensureLanguageModel(options);
  const session = await LanguageModel.create(options);
  const tokenLength = await session.measureInputUsage(options.initialPrompts);
  assert_greater_than(tokenLength, 0);
  assert_true(isValueInRange(session.inputUsage, tokenLength));
  assert_regexp_match(
      await session.prompt([{role: 'system', content: ''}]), kValidAudioRegex);
}, 'Test Audio initialPrompt');

promise_test(async () => {
  await ensureLanguageModel(kAudioOptions);
  const blob = await (await fetch(kValidAudioPath)).blob();
  const session = await createLanguageModel(kAudioOptions);
  const result =
      await session.prompt(messageWithContent(kAudioPrompt, 'audio', blob));
  assert_regexp_match(result, kValidAudioRegex);
}, 'Prompt with Blob audio content');

promise_test(async (t) => {
  await ensureLanguageModel(kAudioOptions);
  const blob = await (await fetch(kValidImagePath)).blob();
  const session = await createLanguageModel(kAudioOptions);
  // TODO(crbug.com/409615288): Expect a TypeError according to the spec.
  return promise_rejects_dom(
      t, 'DataError',
      session.prompt(messageWithContent(kImagePrompt, 'audio', blob)));
}, 'Prompt audio with blob containing invalid audio data.');

promise_test(async () => {
  await ensureLanguageModel(kAudioOptions);
  const audio_data = await fetch(kValidAudioPath);
  const audioCtx = new AudioContext();
  const buffer = await audioCtx.decodeAudioData(await audio_data.arrayBuffer());
  const session = await createLanguageModel(kAudioOptions);
  const result =
      await session.prompt(messageWithContent(kAudioPrompt, 'audio', buffer));
  assert_regexp_match(result, kValidAudioRegex);
}, 'Prompt with AudioBuffer');

promise_test(async () => {
  await ensureLanguageModel(kAudioOptions);
  const audio_data = await fetch(kValidAudioPath);
  const session = await createLanguageModel(kAudioOptions);
  const result = await session.prompt(messageWithContent(
      kAudioPrompt, 'audio', await audio_data.arrayBuffer()));
  assert_regexp_match(result, kValidAudioRegex);
}, 'Prompt with BufferSource - ArrayBuffer');
