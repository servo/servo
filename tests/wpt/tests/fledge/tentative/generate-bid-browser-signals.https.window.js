// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-last

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
