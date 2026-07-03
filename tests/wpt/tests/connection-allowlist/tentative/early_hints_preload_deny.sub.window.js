// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

test(() => {
  navigateToTestCase('preload', 'false', token());
}, 'Early Hints preload to a not-allow-listed url fails.');
