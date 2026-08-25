// META: global=dedicatedworker,sharedworker
// META: script=./resources/evaluation-order-setup.js

// Spec: https://html.spec.whatwg.org/C/#run-a-classic-script
// called from https://html.spec.whatwg.org/C/#import-scripts-into-worker-global-scope
setupTest("importScripts() queueing a microtask then throwing an exception", [
  // Step 6 of #run-a-classic-script.
  "body",

  // Step 7.1.1 ("Clean up after running script") is no-op because JavaScript
  // execution context stack is still non-empty immediately after
  // importScripts() as the outer script is still executing.

  // Step 7.1.2 (Rethrowing an exception) causes worker onerror.
  "global-error",

  // Microtask checkpoint is performed later, perhaps
  // "Clean up after running script" after the outer script is finished.
  "microtask",
]);

importScripts('./resources/evaluation-order-1-throw.js');
