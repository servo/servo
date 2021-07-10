// META: title=StorageManager: estimate() for caches

promise_test(async t => {
  let estimate = await navigator.storage.estimate();

  const cachesUsageBeforeCreate = estimate.usageDetails.caches || 0;

  const cacheName = 'testCache';
  const cache = await caches.open(cacheName);
  t.add_cleanup(() => caches.delete(cacheName));

  await cache.put('/test.json', new Response('x'.repeat(1024*1024)));

  estimate = await navigator.storage.estimate();
  assert_true('caches' in estimate.usageDetails);
  const cachesUsageAfterPut = estimate.usageDetails.caches;
  assert_greater_than(
      cachesUsageAfterPut, cachesUsageBeforeCreate,
      'estimated usage should increase after value is stored');
}, 'estimate() shows usage increase after large value is stored');
