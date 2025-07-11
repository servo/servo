// META: title=Summarizer Create User Activation
// META: script=/resources/testdriver.js
// META: timeout=long

'use strict';

// Mocked model download state may be shared between test cases in the same file
// (see e.g. `EchoAIManagerImpl`), so this test case is kept in a separate file.
// TODO(crbug.com/390246212): Support model state controls for WPTs.
promise_test(async t => {
  // Create requires user activation when availability is 'downloadable'.
  assert_implements_optional(await Summarizer.availability() == 'downloadable');
  assert_false(navigator.userActivation.isActive);
  await promise_rejects_dom(t, 'NotAllowedError', Summarizer.create());
  await test_driver.bless('Summarizer.create', Summarizer.create);

  // Create does not require user activation when availability is 'available'.
  assert_equals(await Summarizer.availability(), 'available');
  assert_false(navigator.userActivation.isActive);
  await Summarizer.create();
}, 'Create requires user activation when availability is "downloadable"');
