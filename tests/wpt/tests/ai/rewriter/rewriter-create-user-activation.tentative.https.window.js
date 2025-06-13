// META: title=Rewriter Create User Activation
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

// Model download state is shared between test cases of the same file when run
// with `EchoAIManagerImpl`, so this test case needs to be on its own file.
promise_test(async t => {
  // Creating Rewriter without user activation rejects with NotAllowedError.
  await promise_rejects_dom(t, 'NotAllowedError', Rewriter.create());

  // Creating Rewriter with user activation succeeds.
  await createRewriter();

  // Expect available after create.
  assert_equals(await Rewriter.availability(), 'available');

  // Now that it is available, we should no longer need user activation.
  await Rewriter.create();
}, 'Rewriter.create() requires user activation when availability is "downloadable"');
