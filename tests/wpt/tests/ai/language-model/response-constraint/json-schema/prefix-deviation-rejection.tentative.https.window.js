// META: title=Language Model Response JSON Schema - Prefix Deviation Rejection
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const badPrefix = 'invalid';
  await promise_rejects_dom(
      t, 'NotSupportedError',
      session.prompt(
          [
            {role: 'user', content: 'hello'},
            {role: 'assistant', content: badPrefix, prefix: true}
          ],
          {responseConstraint: kValidResponseSchema}),
      'Response constraint is not a supported json schema.');
}, 'Prompt should reject if the prefix deviates from the json schema constraint.');
