// META: title=Idle Detection API: Basics

'use strict';

promise_test(async t => {
  let promise = navigator.idle.query();
  assert_equals(promise.constructor, Promise,
                'query() returns a promise');

  let status = await promise;
  assert_true(status instanceof IdleStatus,
              'query() promise resolves to an IdleStatus');

  assert_true(['active', 'idle'].includes(status.state.user),
                'status has a valid user state');
  assert_true(['locked', 'unlocked'].includes(status.state.screen),
                'status has a valid screen state');

}, 'query() basics');

promise_test(async t => {
  let used = false;

  await navigator.idle.query({
    get threshold() {
      used = true;
      return 1;
    }
  });

  assert_true(used, 'query() options "threshold" member was used');
}, 'query() uses threshold property');

promise_test(async t => {
  return promise_rejects(
    t,
    new TypeError,
    navigator.idle.query({threshold: 0}),
    'Threshold of 0 should reject');
}, 'query() throws with invalid threshold (0)');

promise_test(async t => {
  return promise_rejects(
    t,
    new TypeError,
    navigator.idle.query({threshold: null}),
    'Threshold of null should reject');
}, 'query() throws with invalid threshold (null)');

promise_test(async t => {
  return promise_rejects(
    t,
    new TypeError,
    navigator.idle.query({threshold: -1}),
    'Threshold of negative numbers should reject');
}, 'query() throws with invalid threshold (-1)');

promise_test(async t => {
  return promise_rejects(
    t,
    new TypeError,
    navigator.idle.query({threshold: NaN}),
    'Threshold of NaN should reject');
}, 'query() throws with invalid threshold (NaN)');

promise_test(async t => {
  return navigator.idle.query();
}, 'query() uses a default value for the threshold when none is passed');

promise_test(async t => {
  return navigator.idle.query({threshold: undefined});
}, 'query() uses a default value for the threshold');
