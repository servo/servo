// META: title=Language Model Availability
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

// These tests have no availability requirements, they only test the API shape.

promise_test(async () => {
  assert_true(!!LanguageModel);
  assert_equals(typeof LanguageModel.availability, 'function');
}, 'LanguageModel.availability() is defined');

promise_test(async () => {
  const availability = await LanguageModel.availability();
  assert_in_array(availability, kValidAvailabilities);
}, 'LanguageModel.availability() returns a valid value with no options');

promise_test(async (t) => {
  return promise_rejects_js(t, RangeError, LanguageModel.availability({
    expectedInputs: [{type: 'text', languages: ['en-abc-invalid']}]
  }));
}, 'LanguageModel.availability() rejects when given invalid language tags');

promise_test(async () => {
  // An array of plausible test option values.
  const kCreateOptionsSpec = [
    {topK: [undefined, -2, 0, 1, 1.5, 3, 99]},  // Nominally int 1-10+.
    {temperature: [undefined, -0.5, 0, 0.6, 1, 7]},  // Nominally float 0-1.
    {expectedInputs: [undefined, [], [{type: 'text'}],
       [{type: 'text'}, {type: 'audio'}, {type: 'image'}],
       [{type: 'text', languages: ['en', 'ja', 'ko']}],
       [{type: 'audio', languages: ['es']}, {type: 'image', languages: ['fr']}],
    ]},
    {expectedOutputs: [undefined, [], [{type: 'text'}],
       [{type: 'text'}, {type: 'audio'}, {type: 'image'}],
       [{type: 'text', languages: ['en', 'ja', 'ko']}],
       [{type: 'audio', languages: ['es']}, {type: 'image', languages: ['fr']}],
    ]},
    {initialPrompts: [undefined, [], [{role: 'system', content: 'have fun'}],
      [{role: 'system', content: 'have fun'}, {role: 'user', content: 'be good'}],
      [{role: 'system', content: 'be good'}, {role: 'system', content: 'be bad'}],
      [{role: 'system', content: 'have fun'}, {role: 'system', content: 'be bad'}],
    ]},
  ];
  for (const options of generateOptionCombinations(kCreateOptionsSpec)) {
    const availability = await LanguageModel.availability(options);
    assert_in_array(availability, kValidAvailabilities, JSON.stringify(options));
  }
}, 'LanguageModel.availability() returns a valid value with plausible options');
