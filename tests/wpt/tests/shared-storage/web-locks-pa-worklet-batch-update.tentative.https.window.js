// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=/fledge/tentative/resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: script=/shared-storage/resources/util.js
// META: script=/fenced-frame/resources/utils.js
// META: timeout=long

"use strict;"

subsetTest(promise_test, async test => {
  let worklet = await sharedStorage.createWorklet('resources/simple-module.js');

  const ancestor_key = token();
  let url0 = generateURL("/shared-storage/resources/frame0.html",
                         [ancestor_key]);
  let url1 = generateURL("/shared-storage/resources/frame1.html",
                         [ancestor_key]);

  // Override the default resource path, as we are not running within the Fledge
  // repository.
  RESOURCE_PATH = '/fledge/tentative/resources/';

  const pa_uuid = generateUuid(test);

  let biddingLogicURL = createBiddingScriptURL(
    {
      generateBid:
        `
          sharedStorage.batchUpdate([
              new SharedStorageAppendMethod('key', 'a'),
              new SharedStorageAppendMethod('key', 'a')
            ], {withLock: 'lock1'});

          return {};
        `
    });

  let decisionLogicURL = createDecisionScriptURL(pa_uuid);

  // Invoke `selectURL()` to perform the following steps:
  // 1. Acquires the lock.
  // 2. Reads the current value at the given key.
  // 3. Waits for 500ms.
  // 4. Sets the shared storage value to the read value appended with the given letter.
  // 5. Releases the lock.
  //
  // After 100ms, run a Protected Audience auction that starts a worklet that:
  // - Acquires the same named lock.
  // - Executes two `append` methods, each appending the same letter.
  //
  // Expected behavior: After both of them finish, the value at the given key
  // should contain the letter repeated three times.
  //
  // This demonstrates that:
  // 1. The `withLock` option is effective, preventing the `batchUpdate()`
  //    method interfering with the "get and set" operation. If the lock were
  //    not used, the final value would likely be a single letter.
  // 2. `batchUpdate()` correctly executes all `append` methods within the
  //    batch.
  //
  // Note: This test remains valid even if the `batchUpdate()` call happens
  // outside the critical section protected by the lock within the worklet. The
  // test effectively demonstrates mutual exclusion as long as there's a
  // reasonable chance for `batchUpdate()` to occur while the worklet is still
  // running.
  let select_url_result = await worklet.selectURL(
      "get-wait-set-within-lock",
      [{url: url0}, {url: url1}],
      {data: {'key': 'key',
              'lock_name': 'lock1',
              'append_letter': 'a'},
      resolveToConfig: true});

  // Busy wait for 100ms.
  const startWaitTime = Date.now();
  while (Date.now() - startWaitTime < 100) {}

  // Run a Protected Audience auction which triggers `append()` with the same
  // lock and the same letter.
  await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
    test,
    {
      uuid: pa_uuid,
      interestGroupOverrides: {
        name: pa_uuid,
        biddingLogicURL: biddingLogicURL,
      },
      auctionConfigOverrides: {
        decisionLogicURL: decisionLogicURL
      }
    });

  attachFencedFrame(select_url_result, 'opaque-ads');
  const result = await nextValueFromServer(ancestor_key);
  assert_equals(result, "frame1_loaded");

  await verifyKeyValueForOrigin('key', 'aaa', location.origin);

  await deleteKeyForOrigin('key', location.origin);
}, 'Test for batchUpdate() with a batch lock in a Protected Audience Worklet context');
