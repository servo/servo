// META: title=Buckets API: Basic tests for openOrCreate(), keys(), delete().
// META: global=window,worker

'use strict';

// This test is for initial IDL version optimized for debugging.
// Split and add extensive testing once implementation for the endpoints are
// added and method definitions are more defined.
promise_test(async testCase => {
  await navigator.storageBuckets.openOrCreate('bucket_name');
  testCase.add_cleanup(async () => {
    await navigator.storageBuckets.delete('bucket_name');
  });

  const buckets = await navigator.storageBuckets.keys();
  assert_equals(buckets.length, 1);
  assert_equals(buckets[0], 'bucket_name');
}, 'openOrCreate() stores bucket name');

promise_test(async testCase => {
  await navigator.storageBuckets.openOrCreate('bucket_name');
  await navigator.storageBuckets.openOrCreate('bucket_name');
  testCase.add_cleanup(async () => {
    await navigator.storageBuckets.delete('bucket_name');
  });

  const buckets = await navigator.storageBuckets.keys();
  assert_equals(buckets.length, 1);
  assert_equals(buckets[0], 'bucket_name');
}, 'openOrCreate() does not store duplicate bucket name');

promise_test(async testCase => {
  await navigator.storageBuckets.openOrCreate('bucket_name3');
  await navigator.storageBuckets.openOrCreate('bucket_name1');
  await navigator.storageBuckets.openOrCreate('bucket_name2');
  testCase.add_cleanup(async () => {
    await navigator.storageBuckets.delete('bucket_name1');
    await navigator.storageBuckets.delete('bucket_name2');
    await navigator.storageBuckets.delete('bucket_name3');
  });

  const buckets = await navigator.storageBuckets.keys();
  assert_equals(buckets.length, 3);
  assert_equals(buckets[0], 'bucket_name1');
  assert_equals(buckets[1], 'bucket_name2');
  assert_equals(buckets[2], 'bucket_name3');
}, 'keys() lists all stored bucket names alphabetically');

promise_test(async testCase => {
  await navigator.storageBuckets.openOrCreate('bucket_name1');
  await navigator.storageBuckets.openOrCreate('bucket_name2');
  testCase.add_cleanup(async () => {
    await navigator.storageBuckets.delete('bucket_name1');
    await navigator.storageBuckets.delete('bucket_name2');
  });

  let buckets = await navigator.storageBuckets.keys();
  assert_equals(buckets.length, 2);
  assert_equals(buckets[0], 'bucket_name1');
  assert_equals(buckets[1], 'bucket_name2');

  await navigator.storageBuckets.delete('bucket_name1');

  buckets = await navigator.storageBuckets.keys();
  assert_equals(buckets.length, 1);
  assert_equals(buckets[0], 'bucket_name2');
}, 'delete() removes stored bucket name');

promise_test(async testCase => {
  await navigator.storageBuckets.openOrCreate('bucket_name');
  testCase.add_cleanup(async () => {
    await navigator.storageBuckets.delete('bucket_name');
  });

  let buckets = await navigator.storageBuckets.keys();
  assert_equals(buckets.length, 1);
  assert_equals(buckets[0], 'bucket_name');

  await navigator.storageBuckets.delete('does-not-exist');

  buckets = await navigator.storageBuckets.keys();
  assert_equals(buckets.length, 1);
  assert_equals(buckets[0], 'bucket_name');
}, 'delete() does nothing if bucket name does not exist');
