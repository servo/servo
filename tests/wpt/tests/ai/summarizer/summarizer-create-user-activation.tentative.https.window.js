// META: title=Summarizer Create User Activation
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

// Model download state is shared between test cases of the same file when run
// with `EchoAIManagerImpl`, so this test case needs to be on its own file.
promise_test(async t => {
  // Creating Summarizer without user activation rejects with NotAllowedError.
  await promise_rejects_dom(t, 'NotAllowedError', Summarizer.create());

  // Creating Summarizer with user activation succeeds.
  await createSummarizer();

  // Expect available after create.
  assert_equals(await Summarizer.availability(), 'available');

  // Now that it is available, we should no longer need user activation.
  await Summarizer.create();
}, 'Summarizer.create() requires user activation when availability is "downloadable"');
