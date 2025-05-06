// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-last

"use strict";

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


subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  let decisionLogicURL = createDecisionScriptURL(uuid, {
    scoreAd: `
          if (browserSignals.creativeScanningMetadata != 'shady') {
            throw 'Wrong creativeScanningMetadata';
          }
      `
  });

  await joinGroupAndRunBasicFledgeTestExpectingWinner(test, {
    uuid: uuid,
    interestGroupOverrides: {
      ads: [
        {renderURL: createRenderURL(uuid), creativeScanningMetadata: 'shady'}
      ],
    },
    auctionConfigOverrides:
        {decisionLogicURL: decisionLogicURL, sendCreativeScanningMetadata: true}
  });
}, 'ScoreAd browserSignals.creativeScanningMetadata test, no adComponents');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  let decisionLogicURL = createDecisionScriptURL(uuid, {
    scoreAd: `
          if (browserSignals.creativeScanningMetadata != 'shady') {
            throw 'Wrong creativeScanningMetadata';
          }
          let adComponentsCreativeScanningMetadata =
              browserSignals.adComponentsCreativeScanningMetadata;
          if (!Array.isArray(adComponentsCreativeScanningMetadata) ||
              adComponentsCreativeScanningMetadata.length !== 3 ||
              adComponentsCreativeScanningMetadata[0] !== 'c1' ||
              adComponentsCreativeScanningMetadata[1] !== null ||
              adComponentsCreativeScanningMetadata[2] !== 'c4') {
            throw 'Wrong adComponentsCreativeScanningMetadata';
          }
      `
  });
  let biddingLogicURL = createBiddingScriptURL({
    generateBid: `
          return {
            bid: 1,
            render: { url: interestGroup.ads[0].renderURL,
                      width: '100sw',
                      height: '50px' },
            adComponents: [interestGroup.adComponents[0].renderURL,
                           interestGroup.adComponents[1].renderURL,
                           interestGroup.adComponents[3].renderURL]
          };
        `
  });

  await joinGroupAndRunBasicFledgeTestExpectingWinner(test, {
    uuid: uuid,
    interestGroupOverrides: {
      biddingLogicURL: biddingLogicURL,
      ads: [
        {renderURL: createRenderURL(uuid), creativeScanningMetadata: 'shady'}
      ],
      adComponents: [
        {renderURL: 'https://example.org/a', creativeScanningMetadata: 'c1'},
        {renderURL: 'https://example.org/b'},
        {renderURL: 'https://example.org/c', creativeScanningMetadata: 'c3'},
        {renderURL: 'https://example.org/d', creativeScanningMetadata: 'c4'},
      ]
    },
    auctionConfigOverrides:
        {decisionLogicURL: decisionLogicURL, sendCreativeScanningMetadata: true}
  });
}, 'ScoreAd browserSignals.creativeScanningMetadata test, w/adComponents');
