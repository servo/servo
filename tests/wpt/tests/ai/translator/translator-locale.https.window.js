// META: title=Detect english
// META: global=window
// META: timeout=long
// META: script=resources/util.js
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: script=../resources/locale-util.js

'use strict';

function assert_rejects_invalid_expected_input_languages(
    t, method, sourceLanguage, targetLanguage) {
  return promise_rejects_js(
      t, RangeError, method({sourceLanguage, targetLanguage}));
}

function testInvalidLanguagePairs(t, method) {
  const allValidLanguageTags = Object.values(valid_language_tags).flat();
  // Invalid source language.
  for (const sourceLanguage of invalid_language_tags) {
    for (const targetLanguage of allValidLanguageTags) {
      assert_rejects_invalid_expected_input_languages(
          t, method, sourceLanguage, targetLanguage);
    }
  }
  // Invalid target language.
  for (const sourceLanguage of allValidLanguageTags) {
    for (const targetLanguage of invalid_language_tags) {
      assert_rejects_invalid_expected_input_languages(
          t, method, sourceLanguage, targetLanguage);
    }
  }
  // Invalid source and target language
  for (const sourceLanguage of invalid_language_tags) {
    for (const targetLanguage of invalid_language_tags) {
      assert_rejects_invalid_expected_input_languages(
          t, method, sourceLanguage, targetLanguage);
    }
  }
}

promise_test(async t => {
  // We don't need to consume user activation since it should throw a RangeError
  // before it even can check if it needs to consume user activation.
  testInvalidLanguagePairs(t, Translator.create);
}, 'LanguageDetector.create() throws RangeError for invalid language tags');

promise_test(async t => {
  testInvalidLanguagePairs(t, Translator.availability);
}, 'LanguageDetector.availability() throws RangeError for invalid language tags');
