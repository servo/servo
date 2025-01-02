// META: title=Buckets API: Tests for bucket storage policies.
// META: script=resources/util.js
// META: global=window,worker

'use strict';

function sanitizeQuota(quota) {
  return Math.max(1, Math.min(Number.MAX_SAFE_INTEGER, Math.floor(quota)));
}

async function testQuota(storageKeyQuota, quota, name) {
  const safeQuota = sanitizeQuota(quota);
  const bucket = await navigator.storageBuckets.open(name, { quota: safeQuota });
  const estimateQuota = (await bucket.estimate()).quota;
  assert_equals(estimateQuota, Math.min(safeQuota, storageKeyQuota));
}

promise_test(async testCase => {
  await prepareForBucketTest(testCase);

  const storageKeyQuota = (await navigator.storage.estimate()).quota;

  testQuota(storageKeyQuota, 1, 'one');
  testQuota(storageKeyQuota, storageKeyQuota / 4, 'quarter');
  testQuota(storageKeyQuota, storageKeyQuota / 2, 'half');
  testQuota(storageKeyQuota, storageKeyQuota - 1, 'one_less');
  testQuota(storageKeyQuota, storageKeyQuota, 'origin_quota');
  testQuota(storageKeyQuota, storageKeyQuota + 1, 'one_more');
  testQuota(storageKeyQuota, storageKeyQuota * 2, 'twice');
  testQuota(storageKeyQuota, storageKeyQuota * 4, 'four_times');
  testQuota(storageKeyQuota, Number.MAX_SAFE_INTEGER, 'max_safe_int');
}, 'For an individual bucket, the quota is the minimum of the requested quota and the StorageKey quota.');
