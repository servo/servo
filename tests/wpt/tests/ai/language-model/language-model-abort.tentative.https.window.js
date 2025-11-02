// META: title=Language Model Abort
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await testAbortPromise(t, signal => {
    return createLanguageModel({
      signal: signal
    });
  });
}, "Aborting LanguageModel.create().");

promise_test(async t => {
  const session = await createLanguageModel();
  await testAbortPromise(t, signal => {
    return session.clone({
      signal: signal
    });
  });
}, "Aborting LanguageModel.clone().");

promise_test(async t => {
  const session = await createLanguageModel();
  await testAbortPromise(t, signal => {
    return session.prompt(kTestPrompt, { signal: signal });
  });
}, "Aborting LanguageModel.prompt().");

promise_test(async t => {
  const session = await createLanguageModel();
  await testAbortReadableStream(t, signal => {
    return session.promptStreaming(
      kTestPrompt, { signal: signal }
    );
  });
}, "Aborting LanguageModel.promptStreaming().");
