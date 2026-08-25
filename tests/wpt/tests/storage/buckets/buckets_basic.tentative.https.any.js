// META: title=Buckets API: Basic tests for open(), keys(), delete().
// META: script=resources/util.js
// META: global=window,worker

'use strict';

// This test is for initial IDL version optimized for debugging.
// Split and add extensive testing once implementation for the endpoints are
// added and method definitions are more defined.
promise_test(async testCase => {
  await prepareForBucketTest(testCase);

  await navigator.storageBuckets.open('bucket_name3');
  await navigator.storageBuckets.open('bucket_name1');
  await navigator.storageBuckets.open('bucket_name2');

  const buckets = await navigator.storageBuckets.keys();
  assert_array_equals(
      buckets, ['bucket_name1', 'bucket_name2', 'bucket_name3']);
}, 'keys() lists all stored bucket names alphabetically');

promise_test(async testCase => {
  await prepareForBucketTest(testCase);

  await navigator.storageBuckets.open('bucket_name1');
  await navigator.storageBuckets.open('bucket_name2');

  let buckets = await navigator.storageBuckets.keys();
  assert_array_equals(buckets, ['bucket_name1', 'bucket_name2']);

  await navigator.storageBuckets.delete('bucket_name1');

  buckets = await navigator.storageBuckets.keys();
  assert_array_equals(buckets, ['bucket_name2']);
}, 'delete() removes stored bucket name');

promise_test(async testCase => {
  await prepareForBucketTest(testCase);

  await navigator.storageBuckets.open('bucket_name');

  let buckets = await navigator.storageBuckets.keys();
  assert_array_equals(buckets, ['bucket_name']);

  await navigator.storageBuckets.delete('does-not-exist');

  buckets = await navigator.storageBuckets.keys();
  assert_equals(buckets.length, 1);
  assert_equals(buckets[0], 'bucket_name');
}, 'delete() does nothing if bucket name does not exist');
