if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
}

//Content-Security-Policy: connect-src 'none'; cf .headers file
cspViolationUrl = RESOURCES_DIR + "top.txt";

promise_test(function(test) {
  return promise_rejects(test, new TypeError(), fetch(cspViolationUrl));
}, "Fetch is blocked by CSP, got a TypeError");

done();
