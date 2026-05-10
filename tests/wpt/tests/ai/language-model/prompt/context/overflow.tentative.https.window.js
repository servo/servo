// META: title=Language Model Prompt Context Overflow
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const promise = new Promise(resolve => {
    session.addEventListener('contextoverflow', resolve);
  });
  // Make sure there is something to evict.
  const kLongPrompt = kTestPrompt.repeat(10);
  const usage = await session.measureContextUsage(kLongPrompt);
  assert_greater_than(session.contextWindow, usage);
  await session.append(kLongPrompt);
  assert_greater_than(session.contextUsage, 0);
  // Generate a repeated kLongPrompt string that exceeds contextWindow.
  const repeatCount = session.contextWindow / session.contextUsage;
  const promptString = kLongPrompt.repeat(repeatCount);
  // The prompt promise succeeds, while causing older input to be evicted.
  await Promise.all([promise, session.prompt(promptString)]);
}, 'The `contextoverflow` event is fired when overall usage exceeds the context window');
