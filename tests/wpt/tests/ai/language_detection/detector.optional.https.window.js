// META: title=Detect english
// META: global=window
// META: timeout=long
// META: script=resources/util.js
// META: script=/resources/testdriver.js
// META: script=../resources/util.js

'use strict';

async function assert_detects_correct_language(
    detector, input, expectedLanguage) {
  const results = await detector.detect(input);
  // The highest confidence language should be
  assert_equals(results[0].detectedLanguage, expectedLanguage);
}

promise_test(async t => {
  const testInput = {
    af: 'Dit is \'n voorbeeldsin.',
    el: 'Αυτή είναι μια παραδειγματική πρόταση.',
    'el-Latn': 'Aete einai mia paratheiymatike protase.',
    en: 'This is an example sentence.',
    es: 'Esta es una oración de ejemplo.',
    fr: 'Ceci est un exemple de phrase.',
    hi: 'यह एक उदाहरण वाक्य है.',
    'hi-Latn': 'yh ek udaahrn vaaky hai.',
    it: 'Questa è una frase di esempio.',
    ja: 'これは例文です。',
    'ja-Latn': 'Kore wa reibundesu.',
    ko: '이것은 예문입니다.',
    mi: 'He tauira rerenga korero tenei.',
    nl: 'Dit is een voorbeeldzin.',
    ru: 'Это пример предложения.',
    sr: 'Ово је пример реченице.',
    tr: 'Bu bir örnek cümledir.',
    zh: '这是一个例句。',
    zu: 'Lona umusho oyisibonelo.',
  }

  const expectedInputLanguages = Object.keys(testInput);

  const detector = await createLanguageDetector({expectedInputLanguages});

  for (const [language, input] of Object.entries(testInput)) {
    await assert_detects_correct_language(detector, input, language);
  }
}, 'LanguageDetector.detect() detects the correct language');

promise_test(async () => {
  const expectedInputLanguages = ['en', 'es'];
  const detector = await createLanguageDetector({expectedInputLanguages});
  assert_array_equals(detector.expectedInputLanguages, expectedInputLanguages);
  assert_true(Object.isFrozen(detector.expectedInputLanguages));
}, 'Creating LanguageDetector with expectedInputLanguages');


promise_test(async () => {
  const detector = await createLanguageDetector();

  const results = await detector.detect('');
  assert_equals(results.length, 1);

  const [result] = results;
  assert_equals(result.detectedLanguage, 'und');
  assert_equals(result.confidence, 1);
}, 'LanguageDetector.detect() detects empty string');
