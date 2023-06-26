// Abstracts expectations for reuse in different test frameworks.

cls_expect = (watcher, expectation) => {
  watcher.checkExpectation(expectation);
};
