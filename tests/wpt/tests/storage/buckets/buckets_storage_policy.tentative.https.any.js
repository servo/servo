// META: title=Buckets API: Tests for bucket storage policies.
// META: script=resources/util.js
// META: global=window,worker

'use strict';

function sanitizeQuota(quota) {
  return Math.max(1, Math.min(Number.MAX_SAFE_INTEGER, Math.floor(quota)));
}

async function testQuota(quota, name) {
  const safeQuota = sanitizeQuota(quota);
  const bucket = await navigator.storageBuckets.open(name, { quota: safeQuota });
  const estimateQuota = (await bucket.estimate()).quota;
  assert_equals(estimateQuota, safeQuota);
}

promise_test(async testCase => {
  await prepareForBucketTest(testCase);

  const storageKeyQuota = (await navigator.storage.estimate()).quota;

  testQuota(1, 'one');
  testQuota(storageKeyQuota / 4, 'quarter');
  testQuota(storageKeyQuota / 2, 'half');
  testQuota(storageKeyQuota - 1, 'one_less');
  testQuota(storageKeyQuota, 'origin_quota');
}, 'Bucket quota is properly set as long as it is within the storage quota');
