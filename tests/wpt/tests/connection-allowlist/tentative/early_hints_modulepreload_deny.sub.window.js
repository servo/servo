// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

test(() => {
  navigateToTestCase('modulepreload', 'false', token());
}, 'Early Hints modulepreload to a not-allow-listed url fails.');
