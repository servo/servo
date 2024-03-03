// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-4
// META: variant=?5-8
// META: variant=?9-12
// META: variant=?13-16
// META: variant=?17-last

"use strict;"

// These tests focus on the browserSignals argument passed to generateBid().
// Note that "topLevelSeller" is covered by component auction tests,
// "dataVersion" by trusted signals tests, and cross-origin
// "topWindowHostname" and "seller" are covered by cross origin tests.
//
// Some of these tests use the "uuid" for interest group name, to avoid
// joins/bids from previous tests that failed to clean up after themselves
// from affecting results.

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  let expectedBrowserSignals = {
    "topWindowHostname": window.location.hostname,
    "seller": window.location.origin,
    "adComponentsLimit": 40,
    "joinCount": 1,
    "bidCount": 0,
    "prevWinsMs": []
  }
  let biddingLogicURL = createBiddingScriptURL(
      { generateBid:
          `let expectedBrowserSignals = ${JSON.stringify(expectedBrowserSignals)};

          // Can't check this value exactly.
          expectedBrowserSignals.recency = browserSignals.recency;

          // This value may be affected by other recently run tests.
          expectedBrowserSignals.forDebuggingOnlyInCooldownOrLockout =
              browserSignals.forDebuggingOnlyInCooldownOrLockout;

          // Remove deprecated field, if present.
          delete browserSignals.prevWins;

          if (!deepEquals(browserSignals, expectedBrowserSignals))
             throw "Unexpected browserSignals: " + JSON.stringify(browserSignals);`
      });

  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: {name: uuid,
                                 biddingLogicURL: biddingLogicURL}});
}, 'Only expected fields present.');

// Creates a bidding script URL that expects the "joinCount" to be
// "expectedJoinCount".
function createJoinCountBiddingScriptURL(expectedJoinCount) {
  return createBiddingScriptURL(
    { generateBid:
        `if (browserSignals.joinCount !== ${expectedJoinCount})
           throw "Unexpected joinCount: " + browserSignals.joinCount;`
    });
}

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: {name: uuid,
                                 biddingLogicURL: createJoinCountBiddingScriptURL(1)}});

  // Joining again, even with a different script URL, should increase the join count.
  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: {name: uuid,
                                 biddingLogicURL: createJoinCountBiddingScriptURL(2)}});
}, 'browserSignals.joinCount same joining page.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: {name: uuid,
                                 biddingLogicURL: createJoinCountBiddingScriptURL(1)}});

  // Attempt to re-join the same interest group from a different top-level origin.
  // The join count should still be persisted.
  await joinCrossOriginInterestGroupInTopLevelWindow(
      test, uuid, OTHER_ORIGIN1, window.location.origin,
      { name: uuid,
        biddingLogicURL: createJoinCountBiddingScriptURL(2)});

  await runBasicFledgeTestExpectingWinner(test, uuid);
}, 'browserSignals.joinCount different top-level joining origin.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: {name: uuid,
                                 biddingLogicURL: createJoinCountBiddingScriptURL(1)}});

  // Leaving interest group should clear join count.
  await leaveInterestGroup({name: uuid});

  // Check that join count was cleared.
  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: {name: uuid,
                                 biddingLogicURL: createJoinCountBiddingScriptURL(1)}});
}, 'browserSignals.joinCount leave and rejoin.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportTest(
    test, uuid,
    { generateBid:
        `if (browserSignals.recency === undefined)
           throw new Error("Missing recency in browserSignals.")

         if (browserSignals.recency < 0)
           throw new Error("Recency is a negative value.")

         if (browserSignals.recency > 30000)
           throw new Error("Recency is over 30 seconds threshold.")

         if (browserSignals.recency % 100 !== 0)
           throw new Error("Recency is not rounded to multiple of 100 milliseconds.")

         return {'bid': 9,
                 'render': interestGroup.ads[0].renderURL};`,
      reportWin:
        `sendReportTo('${createBidderReportURL(uuid)}');`
    },
    // expectedReportURLs
    [createBidderReportURL(uuid)]
  );
}, 'Check recency in generateBid() is below a certain threshold and rounded ' +
   'to multiple of 100 milliseconds.');

// Creates a bidding script URL that expects the "bidCount" to be
// "expectedBidCount".
function createBidCountBiddingScriptURL(expectedBidCount) {
  return createBiddingScriptURL(
    { generateBid:
        `if (browserSignals.bidCount !== ${expectedBidCount})
           throw "Unexpected bidCount: " + browserSignals.bidCount;`
    });
}

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Running an auction should not increment "bidCount".
  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createBidCountBiddingScriptURL(0)});

  // These auctions would have no winner if the "bidCount" were incremented.
  await runBasicFledgeAuction(test, uuid);
  await runBasicFledgeAuction(test, uuid);
}, 'browserSignals.bidCount not incremented when ad not used.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createBidCountBiddingScriptURL(0) });
  await runBasicFledgeAuctionAndNavigate(test, uuid);
  // Wait for the navigation to trigger reports. "bidCount" should be updated before
  // any reports are sent.
  await waitForObservedRequests(uuid, [createSellerReportURL(uuid)]);

  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createBidCountBiddingScriptURL(1) });
  await runBasicFledgeAuctionAndNavigate(test, uuid);
  // Wait for the navigation to trigger reports. "bidCount" should be updated before
  // any reports are sent.
  await waitForObservedRequests(uuid, [createSellerReportURL(uuid),
                                       createSellerReportURL(uuid)]);

  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createBidCountBiddingScriptURL(2) });
  await runBasicFledgeTestExpectingWinner(test, uuid);
}, 'browserSignals.bidCount incremented when ad used.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Join an interest group and run an auction and navigate to the winning ad,
  // increasing the bid count to 1.
  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createBidCountBiddingScriptURL(0)});
  await runBasicFledgeAuctionAndNavigate(test, uuid);
  // Wait for the navigation to trigger reports. "bidCount" should be updated before
  // any reports are sent.
  await waitForObservedRequests(uuid, [createSellerReportURL(uuid)]);

  await joinCrossOriginInterestGroupInTopLevelWindow(
      test, uuid, OTHER_ORIGIN1, window.location.origin,
      { name: uuid,
        biddingLogicURL: createBidCountBiddingScriptURL(1) });
  await runBasicFledgeTestExpectingWinner(test, uuid);
}, 'browserSignals.bidCount persists across re-join from other top-level origin.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Join an interest group and run an auction and navigate to the winning ad,
  // increasing the bid count to 1.
  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createBidCountBiddingScriptURL(0) });
  await runBasicFledgeAuctionAndNavigate(test, uuid);
  await waitForObservedRequests(uuid, [createSellerReportURL(uuid)]);

  // Leaving interest group should clear "bidCount".
  await leaveInterestGroup({name: uuid});

  // Check that bid count was cleared.
  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createBidCountBiddingScriptURL(0)});
  await runBasicFledgeTestExpectingWinner(test, uuid);
}, 'browserSignals.bidCount leave and rejoin.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createBidCountBiddingScriptURL(0)} );

  // Run two auctions at once, without any navigations.
  // "bidCount" should be 0 for both auctions.
  fencedFrameConfigs =
      await Promise.all([runBasicFledgeTestExpectingWinner(test, uuid),
                         runBasicFledgeTestExpectingWinner(test, uuid)]);

  // Start navigating to both auction winners.
  createAndNavigateFencedFrame(test, fencedFrameConfigs[0]);
  createAndNavigateFencedFrame(test, fencedFrameConfigs[1]);

  // Wait for navigations to have sent reports (and thus to have updated
  // bid counts).
  await waitForObservedRequests(uuid, [createSellerReportURL(uuid),
                                       createSellerReportURL(uuid)]);

  // Check that "bidCount" has increased by 2.
  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createBidCountBiddingScriptURL(2) });
  await runBasicFledgeTestExpectingWinner(test, uuid);
}, 'browserSignals.bidCount two auctions at once.');

// Creates a bidding script URL that expects "prevWinsMs" to be
// "expectedPrevWinsMs". All times in "expectedPrevWinsMs" must be 0.
//
// "adIndex" is the index of the ad to use in the bid.
function createPrevWinsMsBiddingScriptURL(expectedPrevWinsMs, adIndex = 0) {
  return createBiddingScriptURL(
    { generateBid:
        `for (let i = 0; i < browserSignals.prevWinsMs.length; i++) {
           // Check age is in a reasonable range.
           if (browserSignals.prevWinsMs[i][0] < 0 ||
               browserSignals.prevWinsMs[i][0] > 30000) {
             throw "Unexpected prevWinsMs time: " + JSON.stringify(browserSignals.prevWinsMs);
           }

           // Set age to 0.
           browserSignals.prevWinsMs[i][0] = 0;

           // Remove obsolete field, if present.
           delete browserSignals.prevWinsMs[i][1].render_url;
         }
         if (!deepEquals(browserSignals.prevWinsMs, ${JSON.stringify(expectedPrevWinsMs)}))
           throw "Unexpected prevWinsMs: " + JSON.stringify(browserSignals.prevWinsMs);

           return {
             bid: 1,
             render: interestGroup.ads[${adIndex}].renderURL
         };`
    });
}

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Running an auction should not increment "prevWinsMs".
  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createPrevWinsMsBiddingScriptURL([])});

  // These auctions would have no winner if the "prevWinsMs" were incremented.
  await runBasicFledgeAuction(test, uuid);
  await runBasicFledgeAuction(test, uuid);
}, 'browserSignals.prevWinsMs not affected when ad not used.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createPrevWinsMsBiddingScriptURL([]) });
  await runBasicFledgeAuctionAndNavigate(test, uuid);
  // Wait for the navigation to trigger reports. "prevWinsMs" should be updated before
  // any reports are sent.
  await waitForObservedRequests(uuid, [createSellerReportURL(uuid)]);

  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createPrevWinsMsBiddingScriptURL(
            [[0, {renderURL: createRenderURL(uuid)}]]) });
  await runBasicFledgeAuctionAndNavigate(test, uuid);
  // Wait for the navigation to trigger reports. "prevWinsMs" should be updated before
  // any reports are sent.
  await waitForObservedRequests(uuid, [createSellerReportURL(uuid),
                                       createSellerReportURL(uuid)]);

  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createPrevWinsMsBiddingScriptURL(
            [ [0, {renderURL: createRenderURL(uuid)}],
              [0, {renderURL: createRenderURL(uuid)}]]) });
  await runBasicFledgeTestExpectingWinner(test, uuid);
}, 'browserSignals.prevWinsMs, no metadata.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const ads = [ {renderURL: createRenderURL(uuid, 0), metadata: null},
                {renderURL: createRenderURL(uuid, 1), metadata: ['1', 2, {3: 4}]},
                {renderURL: createRenderURL(uuid, 2)} ];

  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createPrevWinsMsBiddingScriptURL([], /*adIndex=*/0),
        ads: ads });
  await runBasicFledgeAuctionAndNavigate(test, uuid);
  await waitForObservedRequests(uuid, [createSellerReportURL(uuid)]);

  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createPrevWinsMsBiddingScriptURL(
            [[0, {renderURL: createRenderURL(uuid, 0), metadata: null}]],
            /*adIndex=*/1),
        ads: ads });
  await runBasicFledgeAuctionAndNavigate(test, uuid);
  await waitForObservedRequests(uuid, [createSellerReportURL(uuid),
                                       createSellerReportURL(uuid)]);

  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createPrevWinsMsBiddingScriptURL(
            [ [0, {renderURL: createRenderURL(uuid, 0), metadata: null}],
              [0, {renderURL: createRenderURL(uuid, 1), metadata: ['1', 2, {3: 4}]}] ],
            /*adIndex=*/2),
        ads: ads });
  await runBasicFledgeAuctionAndNavigate(test, uuid);
  await waitForObservedRequests(uuid, [createSellerReportURL(uuid),
                                       createSellerReportURL(uuid),
                                       createSellerReportURL(uuid)]);

  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createPrevWinsMsBiddingScriptURL(
            [ [0, {renderURL: createRenderURL(uuid, 0), metadata: null}],
              [0, {renderURL: createRenderURL(uuid, 1), metadata: ['1', 2, {3: 4}]}],
              [0, {renderURL: createRenderURL(uuid, 2)}] ]),
        ads: ads });
  await runBasicFledgeTestExpectingWinner(test, uuid);
}, 'browserSignals.prevWinsMs, with metadata.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const ads = [ {renderURL: createRenderURL(uuid, 0), metadata: null},
                {renderURL: createRenderURL(uuid, 1), metadata: ['1', 2, {3: 4}]},
                {renderURL: createRenderURL(uuid, 2)} ];

  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createPrevWinsMsBiddingScriptURL([]),
        ads: [{renderURL: createRenderURL(uuid, 0), metadata: null}] });
  await runBasicFledgeAuctionAndNavigate(test, uuid);
  await waitForObservedRequests(uuid, [createSellerReportURL(uuid)]);

  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createPrevWinsMsBiddingScriptURL(
            [[0, {renderURL: createRenderURL(uuid, 0), metadata: null}]]),
        ads: [{renderURL: createRenderURL(uuid, 1), metadata: ['1', 2, {3: 4}]}] });
  await runBasicFledgeAuctionAndNavigate(test, uuid);
  await waitForObservedRequests(uuid, [createSellerReportURL(uuid),
                                       createSellerReportURL(uuid)]);

  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createPrevWinsMsBiddingScriptURL(
            [ [0, {renderURL: createRenderURL(uuid, 0), metadata: null}],
              [0, {renderURL: createRenderURL(uuid, 1), metadata: ['1', 2, {3: 4}]}] ]),
        ads: [{renderURL: createRenderURL(uuid, 2)}] });
  await runBasicFledgeAuctionAndNavigate(test, uuid);
  await waitForObservedRequests(uuid, [createSellerReportURL(uuid),
                                       createSellerReportURL(uuid),
                                       createSellerReportURL(uuid)]);

  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createPrevWinsMsBiddingScriptURL(
            [ [0, {renderURL: createRenderURL(uuid, 0), metadata: null}],
              [0, {renderURL: createRenderURL(uuid, 1), metadata: ['1', 2, {3: 4}]}],
              [0, {renderURL: createRenderURL(uuid, 2)}] ]) });
  await runBasicFledgeTestExpectingWinner(test, uuid);
}, 'browserSignals.prevWinsMs, different set of ads for each bid.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Join an interest group and run an auction and navigate to the winning ad,
  // which should be logged in "prevWinsMs".
  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createPrevWinsMsBiddingScriptURL([])});
  await runBasicFledgeAuctionAndNavigate(test, uuid);
  await waitForObservedRequests(uuid, [createSellerReportURL(uuid)]);

  await joinCrossOriginInterestGroupInTopLevelWindow(
      test, uuid, OTHER_ORIGIN1, window.location.origin,
      { name: uuid,
        biddingLogicURL: createPrevWinsMsBiddingScriptURL(
            [[0, {renderURL: createRenderURL(uuid)}]]) });
  await runBasicFledgeTestExpectingWinner(test, uuid);
}, 'browserSignals.prevWinsMs persists across re-join from other top-level origin.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Join an interest group and run an auction and navigate to the winning ad,
  // which should be logged in "prevWinsMs".
  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createPrevWinsMsBiddingScriptURL([])});
  await runBasicFledgeAuctionAndNavigate(test, uuid);
  await waitForObservedRequests(uuid, [createSellerReportURL(uuid)]);

  // Leaving interest group should clear "prevWinsMs".
  await leaveInterestGroup({name: uuid});

  // Check that bid count was cleared.
  await joinInterestGroup(
      test, uuid,
      { name: uuid,
        biddingLogicURL: createPrevWinsMsBiddingScriptURL([])});
  await runBasicFledgeTestExpectingWinner(test, uuid);
}, 'browserSignals.prevWinsMs leave and rejoin.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Join an interest group with a WASM helper that exposes a single "increment" method,
  // and make sure that method can be invoked and behaves as expected.
  await joinGroupAndRunBasicFledgeTestExpectingWinner(
    test,
    { uuid: uuid,
      interestGroupOverrides: {
          biddingWasmHelperURL: `${RESOURCE_PATH}wasm-helper.py`,
          biddingLogicURL: createBiddingScriptURL(
            { generateBid:
                `if (!browserSignals.wasmHelper)
                   throw "No WASM helper";

                 let instance = new WebAssembly.Instance(browserSignals.wasmHelper);
                 if (!instance)
                   throw "Couldn't create WASM Instance";

                 if (!deepEquals(Object.keys(instance.exports), ["increment"]))
                   throw "Unexpected exports: " + JSON.stringify(instance.exports);

                 if (instance.exports.increment(1) != 2)
                   throw "Unexpected increment result: " + instance.exports.increment(1);` })
      }
    });
}, 'browserSignals.wasmHelper.');
