// META: title=LanguageModel.create() User Activation Tests
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  assert_implements_optional(
      await LanguageModel.availability() == 'downloadable');
  assert_false(navigator.userActivation.isActive);
  return promise_rejects_dom(
      t, 'NotAllowedError', LanguageModel.create(),
      'LanguageModel.create() should fail without any user activation.');
}, 'Create fails without user activation');

promise_test(async t => {
  assert_implements_optional(
      await LanguageModel.availability() == 'downloadable');

  await test_driver.bless('Enable LanguageModel create()');

  // Consume transient activation.
  const win = window.open('about:blank', '_blank');
  win.close();
  assert_true(navigator.userActivation.hasBeenActive);
  assert_false(navigator.userActivation.isActive);

  return promise_rejects_dom(t, 'NotAllowedError', LanguageModel.create());
}, 'Create fails with sticky activation when availability is "downloadable"');
