// META: title=Writer Create User Activation
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

// Model download state is shared between test cases of the same file when run
// with `EchoAIManagerImpl`, so this test case needs to be on its own file.
promise_test(async t => {
  // Creating Writer without user activation rejects with NotAllowedError.
  await promise_rejects_dom(t, 'NotAllowedError', Writer.create());

  // Creating Writer with user activation succeeds.
  await createWriter();

  // Expect available after create.
  assert_equals(await Writer.availability(), 'available');

  // Now that it is available, we should no longer need user activation.
  await Writer.create();
}, 'Writer.create() requires user activation when availability is "downloadable"');
