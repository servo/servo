// META: title=DecisionModel Create User Activation
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

const kSchema = {
  questions: [{id: 'is_urgent', type: 'boolean', prompt: 'Is this urgent?'}],
};

// Mocked model download state may be shared between test cases in the same file
// (see e.g. `EchoAIManagerImpl`), so this test case is kept in a separate file.
promise_test(async t => {
  // Create requires user activation when availability is 'downloadable'.
  assert_implements_optional(
      await DecisionModel.availability(kSchema) == 'downloadable');
  assert_false(navigator.userActivation.isActive);
  await promise_rejects_dom(
      t, 'NotAllowedError', DecisionModel.create(kSchema));
  await test_driver.bless();
  const createPromise = DecisionModel.create(kSchema);
  // User activation is not consumed by the create call.
  assert_true(navigator.userActivation.isActive);
  consumeTransientUserActivation();
  (await createPromise).destroy();

  // Create does not require transient user activation.
  assert_equals(await DecisionModel.availability(kSchema), 'available');
  assert_false(navigator.userActivation.isActive);
  (await DecisionModel.create(kSchema)).destroy();
}, 'Create requires sticky user activation when availability is "downloadable"');
