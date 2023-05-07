// META: title=Buckets API: Tests for bucket storage policies.
// META: script=/storage/buckets/resources/util.js
// META: global=window,worker

'use strict';

promise_test(async testCase => {
  await prepareForBucketTest(testCase);

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


promise_test(async testCase => {
  await prepareForBucketTest(testCase);

  // IndexedDB
  {
    const quota = 1;
    const bucket = await navigator.storageBuckets.open('idb', {quota});

    const objectStoreName = 'store';
    const db = await indexedDbOpenRequest(
        testCase, bucket.indexedDB, 'db', (db_to_upgrade) => {
          db_to_upgrade.createObjectStore(objectStoreName);
        });

    const overflowBuffer = new Uint8Array(quota + 1);

    const txn = db.transaction(objectStoreName, 'readwrite');
    txn.objectStore(objectStoreName).add('', overflowBuffer);

    await promise_rejects_dom(
        testCase, 'QuotaExceededError', transactionPromise(txn));
  }
}, 'A QuotaExceededError is thrown when a storage API exceeds the quota of the bucket its in.');
