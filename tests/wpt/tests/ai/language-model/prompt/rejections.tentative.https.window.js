// META: title=Language Model Prompt Rejections
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const result1 = session.prompt([
    {role: 'user', content: 'foo'},
    {role: 'system', content: 'bar'},
  ]);
  await promise_rejects_js(t, TypeError, result1);

  const result2 = session.prompt([
    {role: 'system', content: 'foo'},
    {role: 'system', content: 'bar'},
  ]);
  await promise_rejects_js(t, TypeError, result2);

  const result3 = session.prompt({role: 'system', content: 'foo'});
  await promise_rejects_js(
      t, TypeError, session.prompt([{role: 'system', content: 'bar'}]));
  await result3;
}, 'prompt() should reject system role messages after other messages');
