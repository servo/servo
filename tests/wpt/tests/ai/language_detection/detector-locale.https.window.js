// META: title=LanguageDetector locale tests
// META: global=window
// META: timeout=long
// META: script=resources/util.js
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: script=../resources/locale-util.js

'use strict';

function getAvailability(expectedInputLanguages) {
  return LanguageDetector.availability({expectedInputLanguages});
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
  return (await createLanguageDetector({expectedInputLanguages}))
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
    if (await getAvailability([languageSubtag]) === 'unavailable') {
      continue;
    }

    await assert_valid_expected_input_languages(languageSubtag);

    for (const variation of variations) {
      await assert_valid_expected_input_languages(variation);
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

function assert_rejects_invalid_expected_input_languages(
    t, method, expectedInputLanguages) {
  return promise_rejects_js(t, RangeError, method({expectedInputLanguages}));
}

promise_test(async t => {
  for (const languageTag of invalid_language_tags) {
    assert_rejects_invalid_expected_input_languages(
        t, createLanguageDetector, [languageTag]);
  }
  assert_rejects_invalid_expected_input_languages(
      t, createLanguageDetector, invalid_language_tags);
}, 'LanguageDetector.create() throws RangeError for invalid language tags');

promise_test(async t => {
  for (const languageTag of invalid_language_tags) {
    assert_rejects_invalid_expected_input_languages(
        t, LanguageDetector.availability, [languageTag]);
  }
  assert_rejects_invalid_expected_input_languages(
      t, LanguageDetector.availability, invalid_language_tags);
}, 'LanguageDetector.availability() throws RangeError for invalid language tags');
