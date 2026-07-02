// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

test(() => {
  navigateToTestCase('preload', 'true', token());
}, 'Early Hints preload to an allow-listed url succeeds.');
