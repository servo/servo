// META: title=Rewriter measureInputUsage
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const rewriter = await createRewriter();
  const result = await rewriter.measureInputUsage(kTestPrompt);
  assert_greater_than(result, 0);
}, 'Rewriter.measureInputUsage() returns non-empty result');
