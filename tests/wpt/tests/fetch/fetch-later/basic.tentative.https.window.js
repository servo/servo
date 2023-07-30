// META: script=/resources/testharness.js
// META: script=/resources/testharnessreport.js

'use strict';

test(() => {
  assert_throws_js(TypeError, () => fetchLater());
}, `fetchLater() cannot be called without request.`);

test(() => {
  const result = fetchLater('/');
  assert_false(result.sent);
}, `fetchLater()'s return tells the deferred request is not yet sent.`);
