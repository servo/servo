// META: title=Language Model Create Multimodal
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

const kValidImagePath = '/images/computer.jpg';
const kValidAudioPath = '/media/speech.wav';

promise_test(async () => {
  await ensureLanguageModel({expectedInputs: [{type: 'audio'}, {type: 'image'}]});
  const kSupportedCreateOptions = [
    { expectedInputs: [{type: 'audio'}] },
    { expectedInputs: [{type: 'image'}] },
    { expectedInputs: [{type: 'audio'}, {type: 'image'}, {type: 'text'}] },
    { expectedInputs: [{type: 'audio', languages: ['en']}] },
    { expectedInputs: [{type: 'image', languages: ['en']}] },
    { expectedInputs: [{type: 'audio', languages: ['en']},
                       {type: 'image', languages: ['en']},
                       {type: 'text', languages: ['en']}] },
  ];
  for (const options of kSupportedCreateOptions) {
    assert_true(!!await createLanguageModel(options), JSON.stringify(options));
  }
}, 'LanguageModel.create() succeeds with supported multimodal type and language options');

promise_test(async () => {
  await ensureLanguageModel({expectedInputs: [{type: 'audio'}, {type: 'image'}]});
  const audioContent = { type:'audio', value: await (await fetch(kValidAudioPath)).blob() };
  const imageContent = { type:'image', value: await (await fetch(kValidImagePath)).blob() };
  const kSupportedCreateOptions = [
    { expectedInputs: [{type: 'audio'}], initialPrompts: [{role: 'user', content: [audioContent]}] },
    { expectedInputs: [{type: 'image'}], initialPrompts: [{role: 'user', content: [imageContent]}] },
    { expectedInputs: [{type: 'audio'}, {type: 'image'}],
      initialPrompts: [{role: 'user', content: [audioContent, imageContent]}] },
  ];
  for (const options of kSupportedCreateOptions) {
    // TODO(crbug.com/419599702): Ensure the model actually gets initialPrompts.
    assert_true(!!await createLanguageModel(options), JSON.stringify(options));
  }
}, 'LanguageModel.create() succeeds with supported multimodal initialPrompts');

promise_test(async t => {
  await ensureLanguageModel({expectedInputs: [{type: 'audio'}, {type: 'image'}]});
  const audioContent = { type:'audio', value: await (await fetch(kValidAudioPath)).blob() };
  const imageContent = { type:'image', value: await (await fetch(kValidImagePath)).blob() };
  const kUnsupportedCreateOptions = [
    { expectedInputs: [{type: 'audio'}], initialPrompts: [{role: 'user', content: [imageContent]}] },
    { expectedInputs: [{type: 'image'}], initialPrompts: [{role: 'user', content: [audioContent]}] },
  ];
  for (const options of kUnsupportedCreateOptions) {
    await promise_rejects_dom(t, 'NotSupportedError', createLanguageModel(options), JSON.stringify(options));
  }
}, 'LanguageModel.create() fails with unsupported multimodal initialPrompts');
