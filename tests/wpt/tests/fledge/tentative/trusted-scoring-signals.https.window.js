// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-5
// META: variant=?6-10
// META: variant=?11-15
// META: variant=?16-20
// META: variant=?21-25
// META: variant=?26-30
// META: variant=?31-35
// META: variant=?36-40
// META: variant=?41-last

"use strict";

// These tests focus on trustedScoringSignals: Requesting them, handling network
// errors, handling the renderURLs portion of the response, passing renderURLs
// to worklet scripts, and handling the Data-Version header.

// Helper for trusted scoring signals tests. Runs an auction with
// TRUSTED_SCORING_SIGNALS_URL and a single interest group, failing the test
// if there's no winner. "scoreAdCheck" is an expression that should be true
// when evaluated in scoreAd(). "renderURL" can be used to control the response
// given for TRUSTED_SCORING_SIGNALS_URL.
async function runTrustedScoringSignalsTest(test, uuid, renderURL, scoreAdCheck,
                                            additionalInterestGroupOverrides) {
  const auctionConfigOverrides = {
      trustedScoringSignalsURL: TRUSTED_SCORING_SIGNALS_URL,
      decisionLogicURL:
          createDecisionScriptURL(uuid, {
                  scoreAd: `if (!(${scoreAdCheck})) throw "error";` })};
  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test,
      {
        uuid: uuid,
        interestGroupOverrides: {ads: [{ renderURL: renderURL }],
                                 ...additionalInterestGroupOverrides},
        auctionConfigOverrides: auctionConfigOverrides
      });
}

// Much like runTrustedScoringSignalsTest, but runs auctions through reporting
// as well, and evaluates `check` both in scodeAd() and reportResult(). Also
// makes sure browserSignals.dataVersion is undefined in generateBid() and
// reportWin().
async function runTrustedScoringSignalsDataVersionTest(
    test, uuid, renderURL, check) {
  const interestGroupOverrides = {
      biddingLogicURL:
      createBiddingScriptURL({
              generateBid:
                  `if (browserSignals.dataVersion !== undefined)
                      throw "Bad browserSignals.dataVersion"`,
              reportWin:
                  `if (browserSignals.dataVersion !== undefined)
                     sendReportTo('${createSellerReportURL(uuid, '1-error')}');
                   else
                     sendReportTo('${createSellerReportURL(uuid, '1')}');` }),
      ads: [{ renderURL: renderURL }]
  };
  await joinInterestGroup(test, uuid, interestGroupOverrides);

  const auctionConfigOverrides = {
    decisionLogicURL: createDecisionScriptURL(
        uuid,
        { scoreAd:
              `if (!(${check})) return false;`,
          reportResult:
              `if (!(${check}))
                 sendReportTo('${createSellerReportURL(uuid, '2-error')}')
               sendReportTo('${createSellerReportURL(uuid, '2')}')`,
        }),
        trustedScoringSignalsURL: TRUSTED_SCORING_SIGNALS_URL
  }
  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfigOverrides);
  await waitForObservedRequests(
      uuid, [createSellerReportURL(uuid, '1'), createSellerReportURL(uuid, '2')]);
}

// Creates a render URL that, when sent to the trusted-scoring-signals.py,
// results in a trusted scoring signals response with the provided response
// body.
function createScoringSignalsRenderUrlWithBody(uuid, responseBody) {
  return createRenderURL(uuid, /*script=*/null,
                         /*signalsParam=*/`replace-body:${responseBody}`);
}

/////////////////////////////////////////////////////////////////////////////
// Tests where no renderURL value is received for the passed in renderURL.
/////////////////////////////////////////////////////////////////////////////

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const decisionLogicScriptUrl = createDecisionScriptURL(
      uuid,
      { scoreAd: 'if (trustedScoringSignals !== null) throw "error";' });
  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test,
      { uuid: uuid,
        auctionConfigOverrides: { decisionLogicURL: decisionLogicScriptUrl }
      });
}, 'No trustedScoringSignalsURL.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'close-connection');
  await runTrustedScoringSignalsTest(
      test, uuid, renderURL,
      'trustedScoringSignals === null');
}, 'Trusted scoring signals closes the connection without sending anything.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'http-error');
  await runTrustedScoringSignalsTest(test, uuid, renderURL, 'trustedScoringSignals === null');
}, 'Trusted scoring signals response is HTTP 404 error.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'no-content-type');
  await runTrustedScoringSignalsTest(test, uuid, renderURL, 'trustedScoringSignals === null');
}, 'Trusted scoring signals response has no content-type.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'wrong-content-type');
  await runTrustedScoringSignalsTest(test, uuid, renderURL, 'trustedScoringSignals === null');
}, 'Trusted scoring signals response has wrong content-type.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'ad-auction-not-allowed');
  await runTrustedScoringSignalsTest(test, uuid, renderURL, 'trustedScoringSignals === null');
}, 'Trusted scoring signals response does not allow FLEDGE.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'bad-ad-auction-allowed');
  await runTrustedScoringSignalsTest(test, uuid, renderURL, 'trustedScoringSignals === null');
}, 'Trusted scoring signals response has wrong Ad-Auction-Allowed header.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'no-ad-auction-allow');
  await runTrustedScoringSignalsTest(test, uuid, renderURL, 'trustedScoringSignals === null');
}, 'Trusted scoring signals response has no Ad-Auction-Allowed header.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createScoringSignalsRenderUrlWithBody(
      uuid, /*responseBody=*/'');
  await runTrustedScoringSignalsTest(test, uuid, renderURL, 'trustedScoringSignals === null');
}, 'Trusted scoring signals response has no body.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createScoringSignalsRenderUrlWithBody(
      uuid, /*responseBody=*/'Not JSON');
  await runTrustedScoringSignalsTest(test, uuid, renderURL, 'trustedScoringSignals === null');
}, 'Trusted scoring signals response is not JSON.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createScoringSignalsRenderUrlWithBody(
      uuid, /*responseBody=*/'[]');
  await runTrustedScoringSignalsTest(test, uuid, renderURL, 'trustedScoringSignals === null');
}, 'Trusted scoring signals response is a JSON array.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createScoringSignalsRenderUrlWithBody(
      uuid, /*responseBody=*/'{JSON_keys_need_quotes: 1}');
  await runTrustedScoringSignalsTest(test, uuid, renderURL, 'trustedScoringSignals === null');
}, 'Trusted scoring signals response is invalid JSON object.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createScoringSignalsRenderUrlWithBody(
      uuid, /*responseBody=*/'{}');
  await runTrustedScoringSignalsTest(
      test, uuid, renderURL,
      `trustedScoringSignals.renderURL["${renderURL}"] === null`);
}, 'Trusted scoring signals response has no renderURL object.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'no-value');
  await runTrustedScoringSignalsTest(
      test, uuid, renderURL,
      `trustedScoringSignals.renderURL["${renderURL}"] === null`);
}, 'Trusted scoring signals response has no renderURLs.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'wrong-url');
  await runTrustedScoringSignalsTest(
      test, uuid, renderURL,
      `trustedScoringSignals.renderURL["${renderURL}"] === null &&
       Object.keys(trustedScoringSignals.renderURL).length === 1`);
}, 'Trusted scoring signals response has renderURL not in response.');

/////////////////////////////////////////////////////////////////////////////
// Tests where renderURL value is received for the passed in renderURL.
/////////////////////////////////////////////////////////////////////////////

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'null-value');
  await runTrustedScoringSignalsTest(
      test, uuid, renderURL,
      `trustedScoringSignals.renderURL["${renderURL}"] === null`);
}, 'Trusted scoring signals response has null value for renderURL.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'num-value');
  await runTrustedScoringSignalsTest(
      test, uuid, renderURL,
      `trustedScoringSignals.renderURL["${renderURL}"] === 1`);
}, 'Trusted scoring signals response has a number value for renderURL.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null,
      /*signalsParam=*/'string-value');
  await runTrustedScoringSignalsTest(
      test, uuid, renderURL,
      `trustedScoringSignals.renderURL["${renderURL}"] === "1"`);
}, 'Trusted scoring signals response has a string value for renderURL.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'array-value');
  await runTrustedScoringSignalsTest(
      test, uuid, renderURL,
      `JSON.stringify(trustedScoringSignals.renderURL["${renderURL}"]) === '[1,"foo",null]'`);
}, 'Trusted scoring signals response has an array value for renderURL.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'object-value');
  await runTrustedScoringSignalsTest(
      test, uuid, renderURL,
      `Object.keys(trustedScoringSignals.renderURL["${renderURL}"]).length  === 2 &&
       trustedScoringSignals.renderURL["${renderURL}"]["a"] === "b" &&
       JSON.stringify(trustedScoringSignals.renderURL["${renderURL}"]["c"]) === '["d"]'`);
}, 'Trusted scoring signals response has an object value for renderURL.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'+%20 \x00?,3#&');
  await runTrustedScoringSignalsTest(
      test, uuid, renderURL,
      `trustedScoringSignals.renderURL["${renderURL}"] === "default value"`);
}, 'Trusted scoring signals with escaped renderURL.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'hostname');
  await runTrustedScoringSignalsTest(
      test, uuid, renderURL,
      `trustedScoringSignals.renderURL["${renderURL}"] === "${window.location.hostname}"`);
}, 'Trusted scoring signals receives hostname field.');

// Joins two interest groups and makes sure the scoring signals for one are never leaked
// to the seller script when scoring the other.
//
// There's no guarantee in this test that a single request to the server will be made with
// render URLs from two different IGs, though that's the case this is trying to test -
// browsers are not required to support batching, and even if they do, joining any two
// particular requests may be racy.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL1 = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'num-value');
  const renderURL2 = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'string-value');
  await joinInterestGroup(test, uuid, { ads: [{ renderURL: renderURL1 }], name: '1' });
  await joinInterestGroup(test, uuid, { ads: [{ renderURL: renderURL2 }], name: '2' });
  let auctionConfigOverrides = { trustedScoringSignalsURL: TRUSTED_SCORING_SIGNALS_URL };

  // scoreAd() only accepts the first IG's bid, validating its trustedScoringSignals.
  auctionConfigOverrides.decisionLogicURL =
    createDecisionScriptURL(uuid, {
            scoreAd: `if (browserSignals.renderURL === "${renderURL1}" &&
                          trustedScoringSignals.renderURL["${renderURL1}"] !== 1 ||
                          trustedScoringSignals.renderURL["${renderURL2}"] !== undefined)
                        return;` });
  let config = await runBasicFledgeAuction(
      test, uuid, auctionConfigOverrides);
  assert_true(config instanceof FencedFrameConfig,
      `Wrong value type returned from first auction: ${config.constructor.type}`);

  // scoreAd() only accepts the second IG's bid, validating its trustedScoringSignals.
  auctionConfigOverrides.decisionLogicURL =
    createDecisionScriptURL(uuid, {
            scoreAd: `if (browserSignals.renderURL === "${renderURL2}" &&
                          trustedScoringSignals.renderURL["${renderURL1}"] !== undefined ||
                          trustedScoringSignals.renderURL["${renderURL2}"] !== '1')
                        return;` });
  config = await runBasicFledgeAuction(
      test, uuid, auctionConfigOverrides);
  assert_true(config instanceof FencedFrameConfig,
      `Wrong value type returned from second auction: ${config.constructor.type}`);
}, 'Trusted scoring signals multiple renderURLs.');

/////////////////////////////////////////////////////////////////////////////
// Data-Version tests
/////////////////////////////////////////////////////////////////////////////

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid);
  await runTrustedScoringSignalsDataVersionTest(
      test, uuid, renderURL,
      'browserSignals.dataVersion === undefined');
}, 'Trusted scoring signals response has no Data-Version.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, 'data-version:3');
  await runTrustedScoringSignalsDataVersionTest(
      test, uuid, renderURL,
      'browserSignals.dataVersion === 3');
}, 'Trusted scoring signals response has valid Data-Version.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, 'data-version:0');
  await runTrustedScoringSignalsDataVersionTest(
      test, uuid, renderURL,
      'browserSignals.dataVersion === 0');
}, 'Trusted scoring signals response has min Data-Version.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, 'data-version:4294967295');
  await runTrustedScoringSignalsDataVersionTest(
      test, uuid, renderURL,
      'browserSignals.dataVersion === 4294967295');
}, 'Trusted scoring signals response has max Data-Version.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, 'data-version:4294967296');
  await runTrustedScoringSignalsDataVersionTest(
      test, uuid, renderURL,
      'browserSignals.dataVersion === undefined');
}, 'Trusted scoring signals response has too large Data-Version.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, 'data-version:03');
  await runTrustedScoringSignalsDataVersionTest(
      test, uuid, renderURL,
      'browserSignals.dataVersion === undefined');
}, 'Trusted scoring signals response has data-version with leading 0.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, 'data-version:-1');
  await runTrustedScoringSignalsDataVersionTest(
      test, uuid, renderURL,
      'browserSignals.dataVersion === undefined');
}, 'Trusted scoring signals response has negative Data-Version.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, 'data-version:1.3');
  await runTrustedScoringSignalsDataVersionTest(
      test, uuid, renderURL,
      'browserSignals.dataVersion === undefined');
}, 'Trusted scoring signals response has decimal in Data-Version.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, 'data-version:2 2');
  await runTrustedScoringSignalsDataVersionTest(
      test, uuid, renderURL,
      'browserSignals.dataVersion === undefined');
}, 'Trusted scoring signals response has space in Data-Version.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, 'data-version:0x4');
  await runTrustedScoringSignalsDataVersionTest(
      test, uuid, renderURL,
      'browserSignals.dataVersion === undefined');
}, 'Trusted scoring signals response has hex Data-Version.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, 'data-version:3,replace-body:');
  await runTrustedScoringSignalsDataVersionTest(
      test, uuid, renderURL,
      'browserSignals.dataVersion === undefined');
}, 'Trusted scoring signals response has data-version and empty body.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, 'data-version:3,replace-body:[]');
  await runTrustedScoringSignalsDataVersionTest(
      test, uuid, renderURL,
      'browserSignals.dataVersion === undefined');
}, 'Trusted scoring signals response has data-version and JSON array body.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, 'data-version:3,replace-body:{} {}');
  await runTrustedScoringSignalsDataVersionTest(
      test, uuid, renderURL,
      'browserSignals.dataVersion === undefined');
}, 'Trusted scoring signals response has data-version and double JSON object body.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, 'data-version:3,replace-body:{}');
  await runTrustedScoringSignalsDataVersionTest(
      test, uuid, renderURL,
      'browserSignals.dataVersion === 3');
}, 'Trusted scoring signals response has data-version and no renderURLs.');

/////////////////////////////////////////////////////////////////////////////
// Trusted scoring signals + component ad tests.
/////////////////////////////////////////////////////////////////////////////

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'close-connection');
  const componentURL = createRenderURL(uuid, /*script=*/null);
  await runTrustedScoringSignalsTest(
      test, uuid, renderURL,
      `trustedScoringSignals === null`,
      {adComponents: [{ renderURL: componentURL }],
        biddingLogicURL: createBiddingScriptURL({
        generateBid: `return {bid: 1,
                              render: interestGroup.ads[0].renderURL,
                              adComponents: [interestGroup.adComponents[0].renderURL]};`})
      });
}, 'Component ads trusted scoring signals, server closes the connection without sending anything.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'num-value');
  // This should not be sent. If it is, it will take precedence over the "num-value" parameter
  // from "renderURL", resulting in the "renderURL" having a null "trustedScoringSignals" value.
  const componentURL = createScoringSignalsRenderUrlWithBody(
    uuid, /*responseBody=*/'{}');
  await runTrustedScoringSignalsTest(
      test, uuid, renderURL,
      `Object.keys(trustedScoringSignals.renderURL).length === 1 &&
       trustedScoringSignals.renderURL["${renderURL}"] === 1 &&
       trustedScoringSignals.adComponentRenderURLs === undefined`,
      { adComponents: [{ renderURL: componentURL }] });
}, 'Trusted scoring signals request without component ads in bid.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createScoringSignalsRenderUrlWithBody(
    uuid, /*responseBody=*/'{}');
  const componentURL = createRenderURL(uuid, /*script=*/null);
  await runTrustedScoringSignalsTest(
      test, uuid, renderURL,
      `Object.keys(trustedScoringSignals.renderURL).length === 1 &&
       trustedScoringSignals.renderURL["${renderURL}"] === null &&
       Object.keys(trustedScoringSignals.adComponentRenderURLs).length === 1 &&
       trustedScoringSignals.adComponentRenderURLs["${componentURL}"] === null`,
      {adComponents: [{ renderURL: componentURL }],
       biddingLogicURL: createBiddingScriptURL({
       generateBid: `return {bid: 1,
                             render: interestGroup.ads[0].renderURL,
                             adComponents: [interestGroup.adComponents[0].renderURL]};`})
      });
}, 'Component ads trusted scoring signals trusted scoring signals response is empty JSON object.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'hostname');
  const componentURL1 = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'null-value');
  const componentURL2 = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'num-value');
  const componentURL3 = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'string-value');
  const componentURL4 = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'array-value');
  const componentURL5 = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'object-value');
  const componentURL6 = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'wrong-url');

  await runTrustedScoringSignalsTest(
      test, uuid, renderURL,
      `Object.keys(trustedScoringSignals.renderURL).length === 1 &&
       trustedScoringSignals.renderURL["${renderURL}"] === "${window.location.hostname}" &&

       Object.keys(trustedScoringSignals.adComponentRenderURLs).length === 6 &&
       trustedScoringSignals.adComponentRenderURLs["${componentURL1}"] === null &&
       trustedScoringSignals.adComponentRenderURLs["${componentURL2}"] === 1 &&
       trustedScoringSignals.adComponentRenderURLs["${componentURL3}"] === "1" &&
       JSON.stringify(trustedScoringSignals.adComponentRenderURLs["${componentURL4}"]) === '[1,"foo",null]' &&
       Object.keys(trustedScoringSignals.adComponentRenderURLs["${componentURL5}"]).length === 2 &&
       trustedScoringSignals.adComponentRenderURLs["${componentURL5}"]["a"] === "b" &&
       JSON.stringify(trustedScoringSignals.adComponentRenderURLs["${componentURL5}"]["c"]) === '["d"]' &&
       trustedScoringSignals.adComponentRenderURLs["${componentURL6}"] === null`,

      {adComponents: [{ renderURL: componentURL1 }, { renderURL: componentURL2 },
                      { renderURL: componentURL3 }, { renderURL: componentURL4 },
                      { renderURL: componentURL5 }, { renderURL: componentURL6 }],
      biddingLogicURL: createBiddingScriptURL({
          generateBid: `return {bid: 1,
                                render: interestGroup.ads[0].renderURL,
                                adComponents: ["${componentURL1}", "${componentURL2}",
                                               "${componentURL3}", "${componentURL4}",
                                               "${componentURL5}", "${componentURL6}"]};`
      })
    });
}, 'Component ads trusted scoring signals.');
