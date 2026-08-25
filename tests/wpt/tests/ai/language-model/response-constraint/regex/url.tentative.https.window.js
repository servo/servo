// META: title=Language Model Response Regex - URL
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const regex = /^https?:\/\/[^\s$.?#].[^\s]*$/;
  const response =
      await session.prompt('Extract the URL from "Just visit example.com."',
                           {responseConstraint: regex});
  assert_true(typeof response === 'string');
  assert_true(regex.test(response),
              `Response "${response}" should match regex ${regex}`);
}, 'Prompt should work with a URL regex constraint.');
