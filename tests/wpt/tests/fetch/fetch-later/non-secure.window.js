// META: script=/resources/testharness.js
// META: script=/resources/testharnessreport.js

'use strict';

test(() => {
  assert_false(window.hasOwnProperty('fetchLater'));
}, `fetchLater() is not supported in non-secure context.`);
