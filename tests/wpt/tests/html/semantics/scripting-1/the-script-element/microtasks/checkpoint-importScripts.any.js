// META: global=dedicatedworker,sharedworker

// The `then` handlers for `Promise.resolve()` are evaluated in the first
// microtasks checkpoint after `Promise.resolve()`.

// ----------------------------------------------------------------
// Check when microtasks checkpoint is performed around importScripts().

// The expectation is: the `then` handlers are evaluated after the script
// calling importScripts() is finished, not immediately after importScripts().
// Although #clean-up-after-running-script is executed as a part of
// #run-a-classic-script for importScripts()ed scripts, but at that time
// microtasks checkpoint is NOT performed because JavaScript execution context
// stack is not empty.

self.log = [];

// Microtasks should be executed before
// #run-a-classic-script/#run-a-module-script is completed, and thus before
// script evaluation scheduled by setTimeout().
async_test(t => {
  self.addEventListener('error',
      t.unreached_func('error event should not be fired'));

  t.step_timeout(() => {
      assert_array_equals(log, [
          'importScripts()ed script',
          'catch',
          'promise'
        ]);
      t.done();
    },
    0);
}, "Promise resolved during importScripts()");

try {
  importScripts('resources/resolve-then-throw.js');
} catch (e) {
  self.log.push('catch');
}
