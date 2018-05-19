// Only test a subset of tests with, e.g., ?1-10 in the URL.
// Can be used together with <meta name="variant" content="...">
// Sample usage:
// for (const test of tests) {
//   subsetTest(async_test, test.fn, test.name);
// }
(function() {
  var subTestStart = 0;
  var subTestEnd = Infinity;
  var match;
  if (location.search) {
      match = /(?:^\?|&)(\d+)-(\d+|last)(?:&|$)/.exec(location.search);
      if (match) {
        subTestStart = parseInt(match[1], 10);
        if (match[2] !== "last") {
            subTestEnd = parseInt(match[2], 10);
        }
      }
  }
  function shouldRunSubTest(currentSubTest) {
    return currentSubTest >= subTestStart && currentSubTest <= subTestEnd;
  }
  var currentSubTest = 0;
  function subsetTest(testFunc, ...args) {
    currentSubTest++;
    if (shouldRunSubTest(currentSubTest)) {
      return testFunc(...args);
    }
    return null;
  }
  self.shouldRunSubTest = shouldRunSubTest;
  self.subsetTest = subsetTest;
})();
