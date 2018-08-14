// META: title=Web Locks API: navigator.locks.query ordering
// META: script=resources/helpers.js

'use strict';

// Grab a lock and hold until a release function is called. Resolves
// to a release function.
function getLockAndHoldUntilReleased(name, options) {
  let release;
  const promise = new Promise(resolve => { release = resolve; });
  return new Promise(resolve => {
    navigator.locks.request(name, options || {}, lock => {
      resolve(release);
      return promise;
    }).catch(_ => {});
  });
}

promise_test(async t => {
  const res1 = uniqueName(t);
  const res2 = uniqueName(t);
  const res3 = uniqueName(t);

  // These will never be released.
  await Promise.all([
    getLockAndHoldUntilReleased(res1),
    getLockAndHoldUntilReleased(res2),
    getLockAndHoldUntilReleased(res3)
  ]);

  // These requests should be blocked.
  navigator.locks.request(res3, {mode: 'shared'}, lock => {});
  navigator.locks.request(res2, {mode: 'shared'}, lock => {});
  navigator.locks.request(res1, {mode: 'shared'}, lock => {});

  const state = await navigator.locks.query();

  const relevant_pending_names = state.pending.map(lock => lock.name)
                        .filter(name => [res1, res2, res3].includes(name));

  assert_array_equals(relevant_pending_names, [res3, res2, res1],
                      'Pending locks should appear in order.');
}, 'Requests appear in state in order made');

promise_test(async t => {
  const res1 = uniqueName(t);
  const res2 = uniqueName(t);
  const res3 = uniqueName(t);

  // These should be granted, and will be held until released.
  const [release1, release2, release3] = await Promise.all([
    getLockAndHoldUntilReleased(res1),
    getLockAndHoldUntilReleased(res2),
    getLockAndHoldUntilReleased(res3)
  ]);

  // These requests should be blocked.
  const requests = [
    getLockAndHoldUntilReleased(res1),
    getLockAndHoldUntilReleased(res2),
    getLockAndHoldUntilReleased(res3)
  ];

  // Ensure the requests have had a chance to get queued by
  // waiting for something else to make it through the queue.
  await navigator.locks.request(uniqueName(t), lock => {});

  // Now release the previous holders.
  release2();
  release3();
  release1();

  // Wait until the subsequent requests make it through.
  await Promise.all(requests);

  const state = await navigator.locks.query();
  const relevant_held_names = state.held.map(lock => lock.name)
                        .filter(name => [res1, res2, res3].includes(name));

  assert_array_equals(relevant_held_names, [res2, res3, res1],
                      'Held locks should appear in granted order.');
}, 'Held locks appear in state in order granted');

promise_test(async t => {
  const res1 = uniqueName(t);
  const res2 = uniqueName(t);
  const res3 = uniqueName(t);

  // These should be granted, and will be held until stolen.
  await Promise.all([
    getLockAndHoldUntilReleased(res1),
    getLockAndHoldUntilReleased(res2),
    getLockAndHoldUntilReleased(res3)
  ]);

  // Steal in a different order.
  await Promise.all([
    getLockAndHoldUntilReleased(res3, {steal: true}),
    getLockAndHoldUntilReleased(res1, {steal: true}),
    getLockAndHoldUntilReleased(res2, {steal: true})
  ]);

  const state = await navigator.locks.query();
  const relevant_held_names = state.held.map(lock => lock.name)
                        .filter(name => [res1, res2, res3].includes(name));

  assert_array_equals(relevant_held_names, [res3, res1, res2],
                      'Held locks should appear in granted order.');
}, 'Held locks appear in state in order granted, including when stolen');
