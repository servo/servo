// META: title=Buckets API: Tests for the StorageBucket object.
// META: global=window,worker

'use strict';

promise_test(async testCase => {
  const bucket = await navigator.storageBuckets.open('bucket_name');
  testCase.add_cleanup(async () => {
    await navigator.storageBuckets.delete('bucket_name');
  });
  const persisted = await bucket.persisted();
  assert_false(persisted);

  // Also verify that the promise is rejected after the bucket is deleted.
  await navigator.storageBuckets.delete('bucket_name');
  await promise_rejects_dom(testCase, 'UnknownError', bucket.persisted());
}, 'persisted() should default to false');

promise_test(async testCase => {
  const bucket = await navigator.storageBuckets.open('bucket_name');
  testCase.add_cleanup(async () => {
    await navigator.storageBuckets.delete('bucket_name');
  });
  const estimate = await bucket.estimate();
  assert_greater_than(estimate.quota, 0);
  assert_equals(estimate.usage, 0);

  const cacheName = 'attachments';
  const cacheKey = 'receipt1.txt';
  var inboxCache = await bucket.caches.open(cacheName);
  await inboxCache.put(cacheKey, new Response('bread x 2'))

  const estimate2 = await bucket.estimate();
  assert_equals(estimate.quota, estimate2.quota);
  assert_less_than(estimate.usage, estimate2.usage);
}, 'estimate() should retrieve quota usage');

promise_test(async testCase => {
  const bucket = await navigator.storageBuckets.open(
      'bucket_name', { durability: 'strict' });
  testCase.add_cleanup(async () => {
    await navigator.storageBuckets.delete('bucket_name');
  });

  const durability = await bucket.durability();
  assert_equals('strict', durability);

  await navigator.storageBuckets.delete('bucket_name');
  await promise_rejects_dom(testCase, 'UnknownError', bucket.durability());
}, 'durability() should retrieve bucket durability specified during creation');

promise_test(async testCase => {
  const bucket = await navigator.storageBuckets.open('bucket_name');
  testCase.add_cleanup(async () => {
    await navigator.storageBuckets.delete('bucket_name');
  });

  const durability = await bucket.durability();
  assert_equals('relaxed', durability);
}, 'Bucket durability defaults to relaxed');

promise_test(async testCase => {
  const oneYear = 365 * 24 * 60 * 60 * 1000;
  const expiresDate = Date.now() + oneYear;
  const bucket = await navigator.storageBuckets.open(
      'bucket_name', { expires: expiresDate });
  testCase.add_cleanup(async () => {
    await navigator.storageBuckets.delete('bucket_name');
  });

  const expires = await bucket.expires();
  assert_equals(expires, expiresDate);
}, 'expires() should retrieve expires date');

promise_test(async testCase => {
  const bucket = await navigator.storageBuckets.open('bucket_name');
  testCase.add_cleanup(async () => {
    await navigator.storageBuckets.delete('bucket_name');
  });

  const expires = await bucket.expires();
  assert_equals(expires, null);

  await navigator.storageBuckets.delete('bucket_name');
  await promise_rejects_dom(testCase, 'UnknownError', bucket.expires());
}, 'expires() should be defaulted to null');

promise_test(async testCase => {
  const bucket = await navigator.storageBuckets.open('bucket_name');
  testCase.add_cleanup(async () => {
    await navigator.storageBuckets.delete('bucket_name');
  });

  const oneYear = 365 * 24 * 60 * 60 * 1000;
  const expiresDate = Date.now() + oneYear;
  await bucket.setExpires(expiresDate);

  const expires = await bucket.expires();
  assert_equals(expires, expiresDate);

  await navigator.storageBuckets.delete('bucket_name');
  await promise_rejects_dom(testCase, 'UnknownError', bucket.setExpires(expiresDate));
}, 'setExpires() should set bucket expires date');

promise_test(async testCase => {
  const oneDay = 24 * 60 * 60 * 1000;
  const expiresDate = Date.now() + oneDay;
  const bucket = await navigator.storageBuckets.open('bucket_name', {
    expires: expiresDate
  });
  testCase.add_cleanup(async () => {
    await navigator.storageBuckets.delete('bucket_name');
  });
  let expires = await bucket.expires();
  assert_equals(expires, expiresDate);

  const oneYear = 365 * oneDay;
  const newExpiresDate = Date.now() + oneYear;
  await bucket.setExpires(newExpiresDate);

  expires = await bucket.expires();
  assert_equals(expires, newExpiresDate);
}, 'setExpires() should update expires date');

promise_test(async testCase => {
  const bucket = await navigator.storageBuckets.open(
      'bucket_name', { durability: 'strict' });
  testCase.add_cleanup(async () => {
    await navigator.storageBuckets.delete('bucket_name');
  });

  const same_bucket = await navigator.storageBuckets.open('bucket_name');
  const durability = await bucket.durability();
  const other_durability = await same_bucket.durability();
  assert_equals(durability, other_durability);

  // Delete the bucket and remake it.
  await navigator.storageBuckets.delete('bucket_name');
  const remade_bucket = await navigator.storageBuckets.open('bucket_name');
  await promise_rejects_dom(testCase, 'UnknownError', bucket.durability());
  const remade_durability = await remade_bucket.durability();
  assert_not_equals(remade_durability, durability);
}, 'two handles can refer to the same bucket, and a bucket name can be reused after deletion');
