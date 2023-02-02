// META: title=Buckets API: Tests for bucket storage policies.
// META: global=window,worker

'use strict';

promise_test(async testCase => {
  testCase.add_cleanup(async () => {
    const bucketNames = await navigator.storageBuckets.keys();
    for (const bucketName of bucketNames) {
      await navigator.storageBuckets.delete(bucketName);
    }
  });

  await promise_rejects_js(
      testCase, TypeError,
      navigator.storageBuckets.open('negative', {quota: -1}));

  await promise_rejects_js(
      testCase, TypeError, navigator.storageBuckets.open('zero', {quota: 0}));

  await promise_rejects_js(
      testCase, TypeError,
      navigator.storageBuckets.open(
          'above_max', {quota: Number.MAX_SAFE_INTEGER + 1}));
}, 'The open promise should reject with a TypeError when quota is requested outside the range of 1 to Number.MAX_SAFE_INTEGER.');
