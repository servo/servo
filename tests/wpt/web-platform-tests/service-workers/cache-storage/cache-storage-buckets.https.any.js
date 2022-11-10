// META: title=Cache.put
// META: global=window,worker
// META: script=/common/get-host-info.sub.js
// META: script=./resources/test-helpers.js
// META: timeout=long

var test_url = 'https://example.com/foo';
var test_body = 'Hello world!';
const { REMOTE_HOST } = get_host_info();

promise_test(async function(test) {
  var inboxBucket = await navigator.storageBuckets.open('inbox');
  var draftsBucket = await navigator.storageBuckets.open('drafts');

  test.add_cleanup(async function() {
    await navigator.storageBuckets.delete('inbox');
    await navigator.storageBuckets.delete('drafts');
  });

  const cacheName = 'attachments';
  const cacheKey = 'receipt1.txt';

  var inboxCache = await inboxBucket.caches.open(cacheName);
  var draftsCache = await draftsBucket.caches.open(cacheName);

  await inboxCache.put(cacheKey, new Response('bread x 2'))
  await draftsCache.put(cacheKey, new Response('eggs x 1'));

  return inboxCache.match(cacheKey)
      .then(function(result) {
        return result.text();
      })
      .then(function(body) {
        assert_equals(body, 'bread x 2', 'Wrong cache contents');
        return draftsCache.match(cacheKey);
      })
      .then(function(result) {
        return result.text();
      })
      .then(function(body) {
        assert_equals(body, 'eggs x 1', 'Wrong cache contents');
      });
}, 'caches from different buckets have different contents');

promise_test(async function(test) {
  var inboxBucket = await navigator.storageBuckets.open('inbox');

  test.add_cleanup(async function() {
    await navigator.storageBuckets.delete('inbox');
  });

  var caches = inboxBucket.caches;
  var attachments = await caches.open('attachments');
  await attachments.put('receipt1.txt', new Response('bread x 2'));
  var result = await attachments.match('receipt1.txt');
  assert_equals(await result.text(), 'bread x 2');

  await navigator.storageBuckets.delete('inbox');

  return promise_rejects_dom(
      test, 'UnknownError', caches.open('attachments'));
}, 'cache.open promise is rejected when bucket is gone');

done();
