importScripts("/resources/testharness.js");

const queryString = self.location.search;
const urlParams = new URLSearchParams(queryString);
const expectedValue = urlParams.has("gpc", "true");
const workerType = urlParams.get("workerType");
test(function(t) {
  assert_equals(navigator.globalPrivacyControl, expectedValue, "Expected navigator.globalPrivacyControl value is read from the worker");
}, `Expected navigator.globalPrivacyControl value (${expectedValue}) is read from the ${workerType} worker`);

done();
