// META: global=window
'use strict';

promise_test(async t => {
  assert_equals(typeof self.screen.isExtended, 'boolean');
}, 'screen.isExtended is present');

promise_test(async t => {
  let iframe = document.body.appendChild(document.createElement('iframe'));
  assert_equals(typeof iframe.contentWindow.screen.isExtended, 'boolean');
}, 'screen.isExtended is present for attached iframes');
