// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

test(() => {
  navigateToTestCase('preconnect', 'true', token());
}, 'Early Hints preconnect to an allow-listed URL succeeds.');
