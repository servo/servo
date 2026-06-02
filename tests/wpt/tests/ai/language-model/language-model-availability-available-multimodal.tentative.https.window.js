// META: title=Language Model Availability Available Multimodal
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

// These tests depend on some level of model availability, whereas those in
// language-model-api-availability-available.https.window.js have no availability requirements.

promise_test(async () => {
  await ensureLanguageModel({expectedInputs: [{type: 'audio'}]});
}, 'LanguageModel.availability() is available with multimodal audio option');

promise_test(async () => {
  await ensureLanguageModel({expectedInputs: [{type: 'image'}]});
}, 'LanguageModel.availability() is available with multimodal image option');

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
    const availability = await LanguageModel.availability(options);
    assert_in_array(availability, kValidAvailabilities, JSON.stringify(options));
  }
}, 'LanguageModel.availability() returns available with supported multimodal options');

promise_test(async () => {
  await ensureLanguageModel({expectedInputs: [{type: 'audio'}, {type: 'image'}]});
  const kUnsupportedCreateOptions = [
    { expectedInputs: [{type: 'audio', languages: ['unk']}] },  // Language not supported.
    { expectedInputs: [{type: 'image', languages: ['unk']}] },  // Language not supported.
  ];
  for (const options of kUnsupportedCreateOptions) {
    assert_equals(await LanguageModel.availability(options), 'unavailable', JSON.stringify(options));
  }
}, 'LanguageModel.availability() returns unavailable with unsupported multimodal options');
