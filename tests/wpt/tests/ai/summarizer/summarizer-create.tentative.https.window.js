// META: title=Summarizer Create
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  assert_true(!!Summarizer);
  assert_equals(typeof Summarizer.create, 'function');
}, 'Summarizer.create() is defined');

promise_test(async t => {
  // Creating Summarizer without user activation rejects with NotAllowedError.
  await promise_rejects_dom(t, 'NotAllowedError', Summarizer.create());

  // Creating Summarizer with user activation succeeds.
  await createSummarizer();

  // Expect available after create.
  assert_equals(await Summarizer.availability(), 'available');

  // Now that it is available, we should no longer need user activation.
  await Summarizer.create();
}, 'Summarizer.create() requires user activation when availability is "downloadable."');
