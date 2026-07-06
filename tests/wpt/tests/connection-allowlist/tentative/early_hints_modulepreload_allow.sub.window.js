// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

test(() => {
  navigateToTestCase('modulepreload', 'true', token());
}, 'Early Hints modulepreload to an allow-listed url succeeds.');
