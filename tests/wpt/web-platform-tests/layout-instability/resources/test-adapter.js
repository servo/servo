// Abstracts expectations for reuse in different test frameworks.

cls_expect = (watcher, expectation) => {
  assert_equals(watcher.score, expectation.score);
};
