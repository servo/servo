// META: title=Embedder Create Requires User Activation
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

promise_test(async t => {
  // 1. Ensure the browser is currently reporting 'downloadable'
  const availability = await SemanticEmbedder.availability();
  assert_implements_optional(availability === 'downloadable', 'Model should be downloadable');

  // 2. Try to create the embedder WITHOUT a user gesture (clicking the page).
  // This should be rejected by the browser to prevent drive-by downloads.
  await promise_rejects_dom(
    t,
    'NotAllowedError',
    SemanticEmbedder.create()
  );
}, 'Create requires sticky user activation when availability is "downloadable"');

promise_test(async t => {
  // 1. Ensure the browser is currently reporting 'downloadable'
  const availability = await SemanticEmbedder.availability();
  assert_implements_optional(availability === 'downloadable', 'Model should be downloadable');

  // 2. Simulate a user clicking on the page
  await test_driver.bless('activate embedder creation');

  // 3. Now that the user has clicked, create() should be allowed to proceed
  // (and trigger the download in the background).
  const embedder = await SemanticEmbedder.create();
  assert_true(!!embedder, 'Embedder should be successfully created');

}, 'Create succeeds with user activation when availability is "downloadable"');
