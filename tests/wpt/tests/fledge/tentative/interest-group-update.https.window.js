// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-4
// META: variant=?5-9
// META: variant=?10-14
// META: variant=?15-19
// META: variant=?20-last

"use strict;"

// This test repeatedly runs auctions to verify an update. A modified bidding script
// continuously throws errors until it detects the expected change in the interest group
// field. This update then stops the auction cycle.
const makeTestForUpdate = ({
  // Test name
  name,
  // fieldname that is getting updated
  interestGroupFieldName,
  // This is used to check if update has happened.
  expectedValue,
  // This is used to create the update response, by default it will always send
  // back the `expectedValue`. Extra steps to make a deep copy.
  responseOverride = expectedValue,
  // Overrides to the interest group.
  interestGroupOverrides = {},
  // Overrides to the auction config.
  auctionConfigOverrides = {},
}) => {
  subsetTest(promise_test, async test => {
    const uuid = generateUuid(test);
    extraBiddingLogic = ``;

    let replacePlaceholders = (ads) => ads.forEach(element => {
      element.renderURL = element.renderURL.replace(`UUID-PLACEHOLDER`, uuid);
    });

    // Testing 'ads' requires some additional setup due to it's reliance
    // on createRenderURL, specifically the bidding script used checks to make
    // sure the `uuid` is the correct one for the test. We use a renderURL
    // with a placeholder 'UUID-PLACEHOLDER' and make sure to replace it
    // before moving on to the test.
    if (interestGroupFieldName === `ads`) {
      if (interestGroupFieldName in interestGroupOverrides) {
        replacePlaceholders(interestGroupOverrides[interestGroupFieldName]);
      }
      replacePlaceholders(responseOverride);
      replacePlaceholders(expectedValue);
    }
    // When checking the render URL, both the deprecated 'renderUrl' and the updated 'renderURL' might exist
    // in the interest group simultaneously, so this test deletes the 'renderUrl' to ensure a
    // clean comparison with deepEquals.
    if (interestGroupFieldName === `ads` || interestGroupFieldName === `adComponents`) {
      extraBiddingLogic = `
      interestGroup.${interestGroupFieldName}.forEach(element => {
        delete element.renderUrl;
      });`
    }

    let expectedValueJSON = JSON.stringify(expectedValue);
    // When the update has not yet been seen, throw an error which will cause the
    // auction not to have a result.
    interestGroupOverrides.biddingLogicURL = createBiddingScriptURL({
      generateBid: `
      ${extraBiddingLogic}
      if (!deepEquals(interestGroup.${interestGroupFieldName}, ${expectedValueJSON})) {
        throw '${interestGroupFieldName} is ' +
            JSON.stringify(interestGroup.${interestGroupFieldName}) +
            ' instead of ' + '${expectedValueJSON}';
      }`
    });

    let responseBody = {};
    responseBody[interestGroupFieldName] = responseOverride;
    let updateParams = {
      body: JSON.stringify(responseBody),
      uuid: uuid
    };
    interestGroupOverrides.updateURL = createUpdateURL(updateParams);
    await joinInterestGroup(test, uuid, interestGroupOverrides);

    // Run an auction until there's a winner, which means update occurred.
    let auctionResult = await runBasicFledgeAuction(test, uuid, auctionConfigOverrides);
    expectNoWinner(auctionResult);
    while (!auctionResult) {
      auctionResult = await runBasicFledgeAuction(test, uuid, auctionConfigOverrides);
    }
  }, name);
};

// In order to test the update process does not update certain fields, this test uses two interest groups:

// * `failedUpdateGroup`: Receives an invalid update, and will continue to throw errors until the update
//                        occurs (which shouldn't happen). This group will have a high bid to ensure if
//                        there was ever a tie, it would win.
// * `successUpdateGroup`: A hard-coded interest group that receives a update and will signal the change
//                         by throwing an error.

// By tracking render URLs, this test guarantees that only the URL associated with the correct update
// (`goodUpdateRenderURL`) is used, and the incorrect URL (`badUpdateRenderURL`) isn't. The test runs
// auctions repeatedly until the update in `successUpdateGroup` stops an auction from producing a winner.
// It then will run one final auction. If there's still no winner, it can infer that `failedUpdateGroup`
// would have received the update if it were propagating correctly.

// If there was a bug in the implementation, a possible case can occur and manifest as a flaky test.
// In this scenerio with the current structure of the Protected Audience API, the `successUpdateGroup`
// updates, and so does the `failedUpdateGroup`, but the `failedUpdateGroup` update happens significantly
// after the  `successUpdateGroup`'s update. In an effort to combat this, after the while loop we run
// another auction to ensure there is no winner (both cases should throw), but depending how slow the
// update takes, this flaky issue still can **possibly** occur.
const makeTestForNoUpdate = ({
  // Test name
  name,
  // fieldname that is should not be getting updated
  interestGroupFieldName,
  // this is used to create the update response and check if it did not happen.
  responseOverride,
  // Overrides to the auction config.
  auctionConfigOverrides = {},
  // Overrides to the interest group.
  failedUpdateGroup = {},
}) => {
  subsetTest(promise_test, async test => {
    const uuid = generateUuid(test);
    // successUpdateGroup

    // These are used in `successUpdateGroup` in order to get a proper update.
    let successUpdateGroup = {};
    let successUpdateField = `userBiddingSignals`;
    let successUpdateFieldExpectedValue = { 'test': 20 };

    const goodUpdateRenderURL = createTrackerURL(window.location.origin, uuid, 'track_get', 'good_update');
    successUpdateGroup.ads = [{ 'renderURL': goodUpdateRenderURL }];
    successUpdateGroup.biddingLogicURL = createBiddingScriptURL({
      generateBid: `
      if (deepEquals(interestGroup.${successUpdateField}, ${JSON.stringify(successUpdateFieldExpectedValue)})){
        throw '${successUpdateField} has updated and is ' +
            '${JSON.stringify(successUpdateFieldExpectedValue)}.'
      }`,
      bid: 5
    });

    let successResponseBody = {};
    successResponseBody[successUpdateField] = successUpdateFieldExpectedValue;
    let successUpdateParams = {
      body: JSON.stringify(successResponseBody),
      uuid: uuid
    };
    successUpdateGroup.updateURL = createUpdateURL(successUpdateParams);
    await joinInterestGroup(test, uuid, successUpdateGroup);
    ///////////////////////// successUpdateGroup

    // failedUpdateGroup
    const badUpdateRenderURL = createTrackerURL(window.location.origin, uuid, `track_get`, `bad_update`);
    // Name needed so we don't have two IGs with same name.
    failedUpdateGroup.name = failedUpdateGroup.name ? failedUpdateGroup.name : `IG name`
    failedUpdateGroup.ads = [{ 'renderURL': badUpdateRenderURL }];
    failedUpdateGroup.biddingLogicURL = createBiddingScriptURL({
      generateBid: `
      if (!deepEquals(interestGroup.${interestGroupFieldName}, ${JSON.stringify(responseOverride)})){
            throw '${interestGroupFieldName} is as expected: '+
            JSON.stringify(interestGroup.${interestGroupFieldName});
      }`,
      bid: 1000
    });
    let failedResponseBody = {};
    failedResponseBody[interestGroupFieldName] = responseOverride;

    let failedUpdateParams = {
      body: JSON.stringify(failedResponseBody),
      uuid: uuid
    };

    failedUpdateGroup.updateURL = createUpdateURL(failedUpdateParams);
    await joinInterestGroup(test, uuid, failedUpdateGroup);
    ///////////////////////// failedUpdateGroup

    // First result should be not be null, `successUpdateGroup` throws when update is detected so until then,
    // run and observe the requests to ensure only `goodUpdateRenderURL` is fetched.
    let auctionResult = await runBasicFledgeTestExpectingWinner(test, uuid, auctionConfigOverrides);
    while (auctionResult) {
      createAndNavigateFencedFrame(test, auctionResult);
      await waitForObservedRequests(
        uuid,
        [goodUpdateRenderURL, createSellerReportURL(uuid)]);
      await fetch(createCleanupURL(uuid));
      auctionResult = await runBasicFledgeAuction(test, uuid, auctionConfigOverrides);
    }
    // Re-run to ensure null because:
    // `successUpdateGroup` should be throwing since update occurred.
    // `failedUpdateGroup` should be throwing since update did not occur.
    await runBasicFledgeTestExpectingNoWinner(test, uuid, auctionConfigOverrides);
  }, name);
};

// Helper to eliminate rewriting a long call to createRenderURL().
// Only thing to change would be signalParams to differentiate between URLs.
const createTempRenderURL = (signalsParams = null) => {
  return createRenderURL(/*uuid=*/`UUID-PLACEHOLDER`,/*script=*/ null,/*signalParams=*/ signalsParams,/*origin=*/ null);
};

makeTestForUpdate({
  name: 'userBiddingSignals update overwrites everything in the field.',
  interestGroupFieldName: 'userBiddingSignals',
  expectedValue: { 'test': 20 },
  interestGroupOverrides: {
    userBiddingSignals: { 'test': 10, 'extra_value': true },
  }
});

makeTestForUpdate({
  name: 'userBiddingSignals updated multi-type',
  interestGroupFieldName: 'userBiddingSignals',
  expectedValue: { 'test': 20, 5: [1, [false, false, true], 3, 'Hello'] },
  interestGroupOverrides: {
    userBiddingSignals: { 'test': 10 },
  }
});

makeTestForUpdate({
  name: 'userBiddingSignals updated to non object',
  interestGroupFieldName: 'userBiddingSignals',
  expectedValue: 5,
  interestGroupOverrides: {
    userBiddingSignals: { 'test': 10 },
  }
});

makeTestForUpdate({
  name: 'userBiddingSignals updated to null',
  interestGroupFieldName: 'userBiddingSignals',
  expectedValue: null,
  interestGroupOverrides: {
    userBiddingSignals: { 'test': 10 },
  }
});

makeTestForUpdate({
  name: 'trustedBiddingSignalsKeys updated correctly',
  interestGroupFieldName: 'trustedBiddingSignalsKeys',
  expectedValue: ['new_key', 'old_key'],
  interestGroupOverrides: {
    trustedBiddingSignalsKeys: ['old_key'],
  }
});

makeTestForUpdate({
  name: 'trustedBiddingSignalsKeys updated to empty array.',
  interestGroupFieldName: 'trustedBiddingSignalsKeys',
  expectedValue: [],
  interestGroupOverrides: {
    trustedBiddingSignalsKeys: ['old_key'],
  }
});


makeTestForUpdate({
  name: 'trustedBiddingSignalsSlotSizeMode updated to slot-size',
  interestGroupFieldName: 'trustedBiddingSignalsSlotSizeMode',
  expectedValue: 'slot-size',
  interestGroupOverrides: {
    trustedBiddingSignalsKeys: ['key'],
    trustedBiddingSignalsSlotSizeMode: 'none',
  }
});

makeTestForUpdate({
  name: 'trustedBiddingSignalsSlotSizeMode updated to all-slots-requested-sizes',
  interestGroupFieldName: 'trustedBiddingSignalsSlotSizeMode',
  expectedValue: 'all-slots-requested-sizes',
  interestGroupOverrides: {
    trustedBiddingSignalsKeys: ['key'],
    trustedBiddingSignalsSlotSizeMode: 'slot-size',
  }
});

makeTestForUpdate({
  name: 'trustedBiddingSignalsSlotSizeMode updated to none',
  interestGroupFieldName: 'trustedBiddingSignalsSlotSizeMode',
  expectedValue: 'none',
  interestGroupOverrides: {
    trustedBiddingSignalsKeys: ['key'],
    trustedBiddingSignalsSlotSizeMode: 'slot-size',
  }
});

makeTestForUpdate({
  name: 'trustedBiddingSignalsSlotSizeMode updated to unknown, defaults to none',
  interestGroupFieldName: 'trustedBiddingSignalsSlotSizeMode',
  expectedValue: 'none',
  responseOverride: 'unknown-type',
  interestGroupOverrides: {
    trustedBiddingSignalsKeys: ['key'],
    trustedBiddingSignalsSlotSizeMode: 'slot-size',
  }
});

makeTestForUpdate({
  name: 'ads updated from 2 ads to 1.',
  interestGroupFieldName: 'ads',
  expectedValue: [
    { renderURL: createTempRenderURL('new_url1'), metadata: 'test1-new' },
  ],
  interestGroupOverrides: {
    ads: [{ renderURL: createTempRenderURL() },
    { renderURL: createTempRenderURL() }]
  }
});

makeTestForUpdate({
  name: 'ads updated from 1 ad to 2.',
  interestGroupFieldName: 'ads',
  expectedValue: [{ renderURL: createTempRenderURL('new_url1'), metadata: 'test1-new' },
                  { renderURL: createTempRenderURL('new_url2'), metadata: 'test2-new' }],
  interestGroupOverrides: {
    ads: [{ renderURL: createTempRenderURL() }]
  }
});

makeTestForUpdate({
  name: 'adComponents updated from 1 adComponent to 2.',
  interestGroupFieldName: 'adComponents',
  expectedValue: [{ renderURL: createTempRenderURL('new_url1'), metadata: 'test1-new' },
                  { renderURL: createTempRenderURL('new_url2'), metadata: 'test2' }],
  interestGroupOverrides: {
    adComponents: [{ renderURL: createTempRenderURL(), metadata: 'test1' }]
  },
});

makeTestForUpdate({
  name: 'adComponents updated from 2 adComponents to 1.',
  interestGroupFieldName: 'adComponents',
  expectedValue: [{ renderURL: createTempRenderURL('new_url1'), metadata: 'test1-new' }],
  interestGroupOverrides: {
    adComponents: [{ renderURL: createTempRenderURL() },
    { renderURL: createTempRenderURL() }]
  },
});

makeTestForUpdate({
  name: 'executionMode updated to frozen context',
  interestGroupFieldName: 'executionMode',
  expectedValue: 'frozen-context',
  interestGroupOverrides: {
    executionMode: 'compatibility',
  }
});

makeTestForUpdate({
  name: 'executionMode updated to compatibility',
  interestGroupFieldName: 'executionMode',
  expectedValue: 'compatibility',
  interestGroupOverrides: {
    executionMode: 'frozen-context',
  }
});

makeTestForUpdate({
  name: 'executionMode updated to group by origin',
  interestGroupFieldName: 'executionMode',
  expectedValue: 'group-by-origin',
  interestGroupOverrides: {
    executionMode: 'compatibility',
  }
});

makeTestForNoUpdate({
  name: 'executionMode updated with invalid input',
  interestGroupFieldName: 'executionMode',
  responseOverride: 'unknown-type',
});

makeTestForNoUpdate({
  name: 'owner cannot be updated.',
  interestGroupFieldName: 'owner',
  responseOverride: OTHER_ORIGIN1,
  auctionConfigOverrides: {
    interestGroupBuyers: [OTHER_ORIGIN1, window.location.origin]
  }
});

makeTestForNoUpdate({
  name: 'name cannot be updated.',
  interestGroupFieldName: 'name',
  responseOverride: 'new_name',
  failedUpdateGroup: { name: 'name2' },
});

makeTestForNoUpdate({
  name: 'executionMode not updated when unknown type.',
  interestGroupFieldName: 'executionMode',
  responseOverride: 'unknown-type',
  failedUpdateGroup: { executionMode: 'compatibility' },
});

makeTestForNoUpdate({
  name: 'trustedBiddingSignalsKeys not updated when bad value.',
  interestGroupFieldName: 'trustedBiddingSignalsKeys',
  responseOverride: 5,
  failedUpdateGroup: {
    trustedBiddingSignalsKeys: ['key'],
  },
});

