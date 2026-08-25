// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

test(() => {
  navigateToTestCase('preconnect', 'false', token());
}, 'Early Hints preconnect to a not-allow-listed URL fails.');
