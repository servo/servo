// META: title=Rewriter Create User Activation
// META: script=/resources/testdriver.js
// META: timeout=long

'use strict';

// Mocked model download state may be shared between test cases in the same file
// (see e.g. `EchoAIManagerImpl`), so this test case is kept in a separate file.
// TODO(crbug.com/390246212): Support model state controls for WPTs.
promise_test(async t => {
  // Create requires user activation when availability is 'downloadable'.
  assert_implements_optional(await Rewriter.availability() == 'downloadable');
  assert_false(navigator.userActivation.isActive);
  await promise_rejects_dom(t, 'NotAllowedError', Rewriter.create());
  await test_driver.bless('Rewriter.create', Rewriter.create);

  // Create does not require user activation when availability is 'available'.
  assert_equals(await Rewriter.availability(), 'available');
  assert_false(navigator.userActivation.isActive);
  await Rewriter.create();
}, 'Create requires user activation when availability is "downloadable"');
