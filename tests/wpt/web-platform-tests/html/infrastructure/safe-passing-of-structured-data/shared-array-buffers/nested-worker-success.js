"use strict";
importScripts("/resources/testharness.js");
importScripts("resources/test-incrementer.js");

promise_test(t => {
  const worker = new Worker("resources/incrementer-worker.js");

  return testSharingViaIncrementerScript(t, worker, "parent worker", worker, "sub-worker");
}, "postMessaging to a dedicated sub-worker allows them to see each others' modifications");

done();
