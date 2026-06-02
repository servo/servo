// META: title=Translator locale tests
// META: global=window
// META: timeout=long
// META: script=resources/util.js
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: script=../resources/locale-util.js

'use strict';

function getAvailability(sourceLanguage, targetLanguage) {
  return Translator.availability({sourceLanguage, targetLanguage});
}

promise_test(async t => {
  for (const [sourceLanguageSubtag, sourceVariations] of Object.entries(
           valid_language_tags)) {
    for (const [targetLanguageSubtag, targetVariations] of Object.entries(
             valid_language_tags)) {
      const languageSubtagAvailability =
          await getAvailability(sourceLanguageSubtag, targetLanguageSubtag);

      // All variations should be consistent with the language subtag.
      for (const sourceVariation of sourceVariations) {
        for (const targetVariation of targetVariations) {
          assert_availability_consistent(
              await getAvailability(sourceVariation, targetVariation),
              languageSubtagAvailability);
        }
      }
    }
  }
}, 'Translator.availability() is consistent between language tag variations');

async function assert_valid_languages(inSourceLanguage, inTargetLanguage) {
  if (['downloading', 'downloadable'].includes(
          await getAvailability(inSourceLanguage, inTargetLanguage))) {
    await test_driver.bless();
  }

  const {sourceLanguage: outSourceLanguage, targetLanguage: outTargetLanguage} =
      await Translator.create(
          {sourceLanguage: inSourceLanguage, targetLanguage: inTargetLanguage});

  assert_is_variation(inSourceLanguage, outSourceLanguage);
  assert_is_canonical(outSourceLanguage);
  assert_is_variation(inTargetLanguage, outTargetLanguage);
  assert_is_canonical(outTargetLanguage);
}

promise_test(async t => {
  for (const [sourceLanguageSubtag, sourceVariations] of Object.entries(
           valid_language_tags)) {
    for (const [targetLanguageSubtag, targetVariations] of Object.entries(
             valid_language_tags)) {
      if (await getAvailability(sourceLanguageSubtag, targetLanguageSubtag) ===
          'unavailable') {
        continue;
      }

      await assert_valid_languages(sourceLanguageSubtag, targetLanguageSubtag);

      for (const sourceVariation of sourceVariations) {
        for (const targetVariation of targetVariations) {
          await assert_valid_languages(sourceVariation, targetVariation);
        }
      }
    }
  }
}, 'Translator has valid source and target languages');

function assert_rejects_invalid_languages(
    t, method, sourceLanguage, targetLanguage) {
  return promise_rejects_js(
      t, RangeError, method({sourceLanguage, targetLanguage}));
}

function testInvalidLanguagePairs(t, method) {
  const allValidLanguageTags = Object.values(valid_language_tags).flat();
  // Invalid source language.
  for (const sourceLanguage of invalid_language_tags) {
    for (const targetLanguage of allValidLanguageTags) {
      assert_rejects_invalid_languages(
          t, method, sourceLanguage, targetLanguage);
    }
  }
  // Invalid target language.
  for (const sourceLanguage of allValidLanguageTags) {
    for (const targetLanguage of invalid_language_tags) {
      assert_rejects_invalid_languages(
          t, method, sourceLanguage, targetLanguage);
    }
  }
  // Invalid source and target language
  for (const sourceLanguage of invalid_language_tags) {
    for (const targetLanguage of invalid_language_tags) {
      assert_rejects_invalid_languages(
          t, method, sourceLanguage, targetLanguage);
    }
  }
}

promise_test(async t => {
  // We don't need to consume user activation since it should throw a RangeError
  // before it even can check if it needs to consume user activation.
  testInvalidLanguagePairs(t, Translator.create);
}, 'Translator.create() throws RangeError for invalid language tags');

promise_test(async t => {
  testInvalidLanguagePairs(t, Translator.availability);
}, 'Translator.availability() throws RangeError for invalid language tags');
