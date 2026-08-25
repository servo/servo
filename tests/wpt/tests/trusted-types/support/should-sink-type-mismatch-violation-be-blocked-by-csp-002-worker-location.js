const testSetupPolicy = trustedTypes.createPolicy("testSetupPolicy", {
  createScriptURL: s => s });

importScripts(testSetupPolicy.createScriptURL("/resources/testharness.js"));
importScripts(testSetupPolicy.createScriptURL("csp-violations.js"));

// For CSP applying to this file, please refer to
// should-sink-type-mismatch-violation-be-blocked-by-csp-002-worker-location.js.headers

importScripts(testSetupPolicy.createScriptURL("should-sink-type-mismatch-violation-be-blocked-by-csp-location.js"));

done();
