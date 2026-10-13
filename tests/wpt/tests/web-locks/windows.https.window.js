// META: title=Web Locks API: Windows
// META: script=resources/helpers.js

"use strict";

promise_test(async (t) => {
  const { window, lockName } = await openWindowAndAcquireLockInWorker(t);

  // This request should be blocked.
  let lock_granted = false;
  const blocked = navigator.locks.request(lockName, lock => { lock_granted = true; });

  // Verify that we can't get it.
  let available = undefined;
  await navigator.locks.request(
    lockName, {ifAvailable: true}, lock => { available = lock !== null; });
  assert_false(available);
  assert_false(lock_granted);

  // Close the window, after which we should be able to acquire the lock here.
  window.close();
  await blocked;
  assert_true(lock_granted);
}, 'Closed window with worker holding lock');

promise_test(async (t) => {
  const { window, lockName } = await openWindowAndAcquireLockInWorker(t);

  // This request should be blocked.
  let lock_granted = false;
  const blocked = navigator.locks.request(lockName, lock => { lock_granted = true; });

  // Verify that we can't get it.
  let available = undefined;
  await navigator.locks.request(
    lockName, {ifAvailable: true}, lock => { available = lock !== null; });
  assert_false(available);
  assert_false(lock_granted);

  // Reload the window, after which we should be able to acquire the lock here.
  window.location.href = 'resources/window.html?refresh=1';
  await blocked;
  assert_true(lock_granted);
}, 'Refreshed window with worker holding lock');

async function openWindowAndAcquireLockInWorker(t) {
  const lockName = uniqueName(t);
  const window = await new Promise((resolve) => {
    const w = globalThis.window.open("resources/window.html");
    w.addEventListener('load', () => { resolve(w); }, { once: true });
    t.add_cleanup(() => w.close());
    return w;
  });

  const {port1, port2} = new MessageChannel();
  const acquiredLock = new Promise((resolve) => {
    port1.onmessage = (_) => resolve();
  });

  // Make popup acquire a lock in a worker.
  window.postMessage(
    { port: port2, worker: {op: 'request', name: lockName, mode: 'exclusive'}},
    '*',
    [port2]
  );
  await acquiredLock;
  return { window, lockName };
}
