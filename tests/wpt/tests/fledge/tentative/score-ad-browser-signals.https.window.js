// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long

"use strict;"

// These tests focus on the browserSignals argument passed to scoreAd().

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  let biddingLogicURL = createBiddingScriptURL(
    {
      generateBid:
        `
          return {
            bid: 1,
            render: { url: interestGroup.ads[0].renderURL,
                      width: '100sw',
                      height: '50px' }
          };
        `
    });

  let decisionLogicURL = createDecisionScriptURL(uuid,
    {
      scoreAd:
        `
          if (!browserSignals.hasOwnProperty('renderSize')) {
            throw 'Missing renderSize member in browserSignals.';
          }
          if (browserSignals.renderSize.width !== '100sw' ||
              browserSignals.renderSize.height !== '50px') {
            throw 'Incorrect renderSize width or height.';
          }
      `
    }
  );

  await joinGroupAndRunBasicFledgeTestExpectingWinner(
    test,
    {
      uuid: uuid,
      interestGroupOverrides: {
        name: uuid,
        biddingLogicURL: biddingLogicURL,
        ads: [{ renderURL: createRenderURL(uuid), sizeGroup: 'group1' }],
        adSizes: { 'size1': { width: '100sw', height: '50px' } },
        sizeGroups: { 'group1': ['size1'] }
      },
      auctionConfigOverrides: {
        decisionLogicURL: decisionLogicURL
      }
    });
}, 'ScoreAd browserSignals renderSize test.');
