const testSetupPolicy = trustedTypes.createPolicy("p", { createScriptURL: s => s });

importScripts(testSetupPolicy.createScriptURL("/resources/testharness.js"));
importScripts(testSetupPolicy.createScriptURL("helper.sub.js"));
importScripts(testSetupPolicy.createScriptURL("csp-violations.js"));

importScripts(testSetupPolicy.createScriptURL(
  "trusted-types-reporting-for-eval.js"
));

done();
