// META: title=Translator Create User Activation
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

// Mocked model download state may be shared between test cases in the same file
// (see e.g. `EchoAIManagerImpl`), so this test case is kept in a separate file.
// TODO(crbug.com/390246212): Support model state controls for WPTs.
promise_test(async t => {
  const languagePair = {sourceLanguage: 'en', targetLanguage: 'ja'};
  const availability = await Translator.availability(languagePair);
  assert_implements_optional(
      availability !== 'unavailable',
      'Translator is not available for the given options');

  // The model may already be downloaded, e.g. because it was side-loaded, or
  // because an earlier test downloaded it. Only assert on the user activation
  // requirement when there is actually something left to download.
  if (availability !== 'available') {
    assert_false(navigator.userActivation.isActive);
    await promise_rejects_dom(t, 'NotAllowedError',
                              Translator.create(languagePair));
    await test_driver.bless();
    await Translator.create(languagePair);
  }

  // Create does not require user activation when availability is 'available'.
  assert_equals(await Translator.availability(languagePair), 'available');
  // The create call above consumes the transient user activation, but consume
  // it explicitly so that this does not rely on which branch was taken.
  consumeTransientUserActivation();
  assert_false(navigator.userActivation.isActive);
  await Translator.create(languagePair);
}, 'Create requires user activation when availability is "downloadable"');
