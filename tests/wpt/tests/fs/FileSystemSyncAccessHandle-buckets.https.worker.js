importScripts('/resources/testharness.js');
importScripts('/storage/buckets/resources/util.js');

'use strict';

// Runs `func` with a sync access handle to an empty file in a new bucket with
// the given quota. The handle is closed before the bucket is cleaned up.
function bucket_sync_access_handle_test(quota, func, description) {
  promise_test(async t => {
    await prepareForBucketTest(t);
    const bucket = await navigator.storageBuckets.open('inbox', {quota});
    const rootDir = await bucket.getDirectory();
    const fileHandle =
        await rootDir.getFileHandle('sync-access-handle.txt', {create: true});
    const handle = await fileHandle.createSyncAccessHandle();
    try {
      func(t, handle);
    } finally {
      handle.close();
    }
  }, description);
}

bucket_sync_access_handle_test(3 * 1024 * 1024, (t, handle) => {
  const buffer = new Uint8Array(2 * 1024 * 1024);
  assert_equals(handle.write(buffer, {at: 0}), buffer.byteLength);
  assert_equals(handle.getSize(), buffer.byteLength);
}, 'A write through a sync access handle within the bucket quota succeeds');

bucket_sync_access_handle_test(3 * 1024 * 1024, (t, handle) => {
  const buffer = new Uint8Array(4 * 1024 * 1024);
  assert_throws_quotaexceedederror(() => handle.write(buffer, {at: 0}), null,
                                   null);
}, 'Bucket quota applies to writes through a sync access handle');

done();
