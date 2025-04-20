// META: title=Detect english
// META: global=window
// META: script=../resources/util.js
// META: script=../resources/locale-util.js

'use strict';

function getAvailability(expectedInputLanguages) {
  return LanguageDetector.availability({expectedInputLanguages});
}

function assert_availability_consistent(
    language_subtag_availability, base_availability) {
  if (base_availability == 'unavailable') {
    // If the language subtag is not available then no variation of it should
    // be available.
    assert_equals(language_subtag_availability, 'unavailable');
  } else {
    // If the language subtag is available, then it definitely shouldn't be
    // unavailable since whatever backing it has could support any variation of
    // it. A variation could have a different availability if a more specific
    // backing is required.
    assert_in_array(
        language_subtag_availability,
        ['downloadable', 'downloading', 'available']);
  }
}

promise_test(async t => {
  for (const [languageSubtag, variations] of Object.entries(
           valid_language_tags)) {
    const languageSubtagAvailability = await getAvailability([languageSubtag]);

    // Test each variation individually.
    for (const variation of variations) {
      assert_availability_consistent(
          await getAvailability([variation]), languageSubtagAvailability);
    }

    // Test all variations.
    assert_availability_consistent(
        await getAvailability(variations), languageSubtagAvailability);
  }
}, 'LanguageDetector.availability() is consistent between language tag variations');


async function getExpectedInputLanguages(expectedInputLanguages) {
  return (await LanguageDetector.create({expectedInputLanguages}))
      .expectedInputLanguages;
}

async function assert_valid_expected_input_languages(language) {
  const expectedInputLanguages = await getExpectedInputLanguages([language]);
  assert_equals(expectedInputLanguages.length, 1);
  assert_is_variation(language, expectedInputLanguages[0]);
  assert_is_canonical(expectedInputLanguages[0]);
}

function uniqueCount(array) {
  return (new Set(array)).size;
}

promise_test(async t => {
  for (const [languageSubtag, variations] of Object.entries(
           valid_language_tags)) {
    await assert_valid_expected_input_languages(languageSubtag)

    for (const variation of variations) {
      await assert_valid_expected_input_languages(variation)
    }

    const expectedInputLanguages = await getExpectedInputLanguages(variations);

    // There should be no duplicates.
    assert_equals(
        expectedInputLanguages.length, uniqueCount(expectedInputLanguages));

    for (const language of expectedInputLanguages) {
      assert_is_canonical(language);
      assert_is_variation(language, languageSubtag);
    }
  }
}, 'LanguageDetector has valid expectedInputLanguages');
