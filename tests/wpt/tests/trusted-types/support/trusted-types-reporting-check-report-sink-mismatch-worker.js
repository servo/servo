const testSetupPolicy = trustedTypes.createPolicy("p", { createScriptURL: s => s });

importScripts(testSetupPolicy.createScriptURL("/resources/testharness.js"));

importScripts(testSetupPolicy.createScriptURL("trusted-types-reporting-check-report-sink-mismatch.js"));

done();
