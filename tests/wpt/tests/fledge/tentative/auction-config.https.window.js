// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
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
// META: variant=?40-45
// META: variant=?46-50
// META: variant=?51-55
// META: variant=?56-60
// META: variant=?61-65
// META: variant=?66-70
// META: variant=?71-last

"use strict;"

// The tests in this file focus on calls to runAdAuction with various
// auctionConfigs.

// We handle promise rejections ourselves.
setup({ allow_uncaught_exception: true });

// Helper for when we expect it to happen.
const interceptUnhandledRejection = () => {
  let invokePromiseResolved;
  let eventHandler = event => {
    event.preventDefault();
    invokePromiseResolved(event.reason);
  }
  window.addEventListener("unhandledrejection", eventHandler, {once: true});
  return new Promise((resolved) => {
    invokePromiseResolved = resolved;
  });
}

// Helper for when we expect it to not happen. This relies on the event
// dispatching being sync.
const unexpectedUnhandledRejection = () => {
  let o = { sawError : false }
  window.addEventListener("unhandledrejection", event => {
    o.sawError = true;
  }, {once: true});
  return o;
}

const makeTest = ({
  // Test name
  name,
  // Expectation function (EXPECT_NULL, etc.)
  expect,
  // Overrides to the auction config.
  auctionConfigOverrides = {},
  // Expectation for a promise error.
  expectPromiseError,
}) => {
  subsetTest(promise_test, async test => {
    let waitPromiseError, dontExpectPromiseError;
    if (expectPromiseError) {
      waitPromiseError = interceptUnhandledRejection();
    } else {
      dontExpectPromiseError = unexpectedUnhandledRejection();
    }

    const uuid = generateUuid(test);
    // Join an interest group so the auction actually runs.
    await joinInterestGroup(test, uuid);
    let auctionResult;
    try {
      auctionResult = await runBasicFledgeAuction(test, uuid, auctionConfigOverrides);
    } catch (e) {
      auctionResult = e;
    }
    expect(auctionResult);

    if (expectPromiseError) {
      expectPromiseError(await waitPromiseError);
    } else {
      assert_false(dontExpectPromiseError.sawError,
                   "Should not see a promise error");
    }
  }, name);
};

// Expect an unsuccessful auction (yielding null).
const EXPECT_NO_WINNER = auctionResult => {
  assert_equals(auctionResult, null, 'Auction unexpected had a winner');
};

// Expect a winner (FencedFrameConfig).
const EXPECT_WINNER =
    auctionResult => {
      assert_true(
          auctionResult instanceof FencedFrameConfig,
          'Auction did not return expected FencedFrameConfig');
    }

// Expect an exception of the given type.
const EXPECT_EXCEPTION = exceptionType => auctionResult => {
  assert_not_equals(auctionResult, null, "got null instead of expected error");
  assert_true(auctionResult instanceof Error, "did not get expected error: " + auctionResult);
  assert_throws_js(exceptionType, () => { throw auctionResult; });
};

const EXPECT_PROMISE_ERROR = auctionResult => {
  assert_not_equals(auctionResult, null, "got null instead of expected error");
  assert_true(auctionResult instanceof TypeError,
              "did not get expected error type: " + auctionResult);
}

makeTest({
  name: 'deprecatedRenderURLReplacements without end bracket is invalid.',
  expect: EXPECT_PROMISE_ERROR,
  expectPromiseError: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {deprecatedRenderURLReplacements: {'${No_End_Bracket': 'SSP'}}
});

makeTest({
  name: 'deprecatedRenderURLReplacements without percents and brackets.',
  expect: EXPECT_PROMISE_ERROR,
  expectPromiseError: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {deprecatedRenderURLReplacements: {'No_Wrapper': 'SSP'}}
});

makeTest({
  name: 'deprecatedRenderURLReplacements without dollar sign.',
  expect: EXPECT_PROMISE_ERROR,
  expectPromiseError: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {deprecatedRenderURLReplacements: {'{No_Dollar_Sign}': 'SSP'}}
});

makeTest({
  name: 'deprecatedRenderURLReplacements without start bracket is invalid.',
  expect: EXPECT_PROMISE_ERROR,
  expectPromiseError: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {deprecatedRenderURLReplacements: {'$No_Start_Bracket}': 'SSP'}}
});

makeTest({
  name: 'deprecatedRenderURLReplacements mix and match is invalid.',
  expect: EXPECT_PROMISE_ERROR,
  expectPromiseError: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {deprecatedRenderURLReplacements: {'${Bracket_And_Percent%%': 'SSP'}}
});

makeTest({
  name: 'deprecatedRenderURLReplacements missing start percent is invalid.',
  expect: EXPECT_PROMISE_ERROR,
  expectPromiseError: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {deprecatedRenderURLReplacements: {'%Missing_Start_Percents%%': 'SSP'}}
});

makeTest({
  name: 'deprecatedRenderURLReplacements single percents is invalid.',
  expect: EXPECT_PROMISE_ERROR,
  expectPromiseError: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {deprecatedRenderURLReplacements: {'%Single_Percents%': 'SSP'}}
});

makeTest({
  name: 'deprecatedRenderURLReplacements without end percents is invalid.',
  expect: EXPECT_PROMISE_ERROR,
  expectPromiseError: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {deprecatedRenderURLReplacements: {'%%No_End_Percents': 'SSP'}}
});

makeTest({
  name: 'sellerRealTimeReportingConfig has default local reporting type',
  expect:  EXPECT_WINNER,
  auctionConfigOverrides: {sellerRealTimeReportingConfig:
                            {type: 'default-local-reporting'}}
});

makeTest({
  name: 'sellerRealTimeReportingConfig has no type',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {sellerRealTimeReportingConfig:
                            {notType: 'default-local-reporting'}}
});

makeTest({
  name: 'sellerRealTimeReportingConfig has unknown type',
  expect:  EXPECT_WINNER,
  auctionConfigOverrides: {sellerRealTimeReportingConfig: {type: 'unknown type'}}
});

makeTest({
  name: 'perBuyerRealTimeReportingConfig',
  expect: EXPECT_WINNER,
  auctionConfigOverrides: {perBuyerRealTimeReportingConfig:
                            {'https://example.com': {type: 'default-local-reporting'}}}
});

makeTest({
  name: 'perBuyerRealTimeReportingConfig has invalid buyer',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {perBuyerRealTimeReportingConfig:
                            {'http://example.com': {type: 'default-local-reporting'}}}
});

makeTest({
  name: 'perBuyerRealTimeReportingConfig has no type',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {perBuyerRealTimeReportingConfig:
                            {'https://example.com': {notType: 'default-local-reporting'}}}
});

makeTest({
  name: 'perBuyerRealTimeReportingConfig has unknown type',
  expect: EXPECT_WINNER,
  auctionConfigOverrides: {perBuyerRealTimeReportingConfig:
                            {'https://example.com': {type: 'unknown type'}}}
});

makeTest({
  name: 'perBuyerRealTimeReportingConfig has no entry',
  expect: EXPECT_WINNER,
  auctionConfigOverrides: {perBuyerRealTimeReportingConfig: {}}
});

makeTest({
  name: 'no buyers => no winners',
  expect: EXPECT_NO_WINNER,
  auctionConfigOverrides: {interestGroupBuyers: []},
});

makeTest({
  name: 'seller is not an https URL',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {seller: "ftp://not-https"},
});

makeTest({
  name: 'decisionLogicURL is invalid',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: { decisionLogicURL: "https://foo:99999999999" },
});

makeTest({
  name: 'decisionLogicURL is cross-origin with seller',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: { decisionLogicURL: "https://example.com" },
});

makeTest({
  name: 'trustedScoringSignalsURL is invalid',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: { trustedScoringSignalsURL: "https://foo:99999999999" },
});

makeTest({
  name: 'valid trustedScoringSignalsURL',
  expect: EXPECT_WINNER,
  auctionConfigOverrides:
      {trustedScoringSignalsURL: window.location.origin + '/resource.json'}
});

makeTest({
  name: 'trustedScoringSignalsURL should not have a fragment',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides:
      {trustedScoringSignalsURL: window.location.origin + '/resource.json#foo'}
});

makeTest({
  name: 'trustedScoringSignalsURL with an empty fragment is not OK',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides:
      {trustedScoringSignalsURL: window.location.origin + '/resource.json#'}
});

makeTest({
  name: 'trustedScoringSignalsURL should not have a query',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides:
      {trustedScoringSignalsURL: window.location.origin + '/resource.json?foo'}
});

makeTest({
  name: 'trustedScoringSignalsURL with an empty query is not OK',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides:
      {trustedScoringSignalsURL: window.location.origin + '/resource.json?'}
});

makeTest({
  name: 'trustedScoringSignalsURL should not have embedded credentials',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {
    trustedScoringSignalsURL: (window.location.origin + '/resource.json')
                                  .replace('https://', 'https://user:pass@')
  }
});

// Cross-origin trustedScoringSignalsURL is fine, but it needs extra
// headers to actually make it work. The auction here doesn't actually
// care if the signals don't load.
makeTest({
  name: 'trustedScoringSignalsURL is cross-origin with seller',
  expect: EXPECT_WINNER,
  auctionConfigOverrides: { trustedScoringSignalsURL: "https://example.com" },
});

makeTest({
  name: 'interestGroupBuyer is invalid',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: { interestGroupBuyers: ["https://foo:99999999999"] },
});

makeTest({
  name: 'interestGroupBuyer is not https',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: { interestGroupBuyers: ["http://example.com"] },
});

makeTest({
  name: 'only one interestGroupBuyer is invalid',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {
    interestGroupBuyers: ["https://example.com", "https://foo:99999999999"],
  },
});

makeTest({
  name: 'only one interestGroupBuyer is not https',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {
    interestGroupBuyers: ["https://example.com", "http://example.com"],
  },
});

makeTest({
  name: 'auctionSignals is invalid as JSON',
  expect: EXPECT_PROMISE_ERROR,
  expectPromiseError: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: { auctionSignals: { sig: BigInt(13) } },
});

makeTest({
  name: 'sellerSignals is invalid as JSON',
  expect: EXPECT_PROMISE_ERROR,
  expectPromiseError: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: { sellerSignals: { sig: BigInt(13) } },
});

makeTest({
  name: 'directFromSellerSignals is invalid',
  expect: EXPECT_PROMISE_ERROR,
  expectPromiseError: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: { directFromSellerSignals: "https://foo:99999999999" },
});

makeTest({
  name: 'directFromSellerSignals is cross-origin with seller',
  expect: EXPECT_PROMISE_ERROR,
  expectPromiseError: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: { directFromSellerSignals: "https://example.com" },
});

makeTest({
  name: 'directFromSellerSignals has nonempty query',
  expect: EXPECT_PROMISE_ERROR,
  expectPromiseError: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: { directFromSellerSignals: window.location.origin + "?foo=bar" },
});

makeTest({
  name: 'perBuyerSignals has invalid URL in a key',
  expect: EXPECT_PROMISE_ERROR,
  expectPromiseError: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: { perBuyerSignals: { "https://foo:99999999999" : {} }},
});

makeTest({
  name: 'perBuyerSignals value is invalid as JSON',
  expect: EXPECT_PROMISE_ERROR,
  expectPromiseError: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {
    perBuyerSignals: { "https://example.com" : { sig: BigInt(1) },
  }},
});

makeTest({
  name: 'perBuyerGroupLimits has invalid URL in a key',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: { perBuyerGroupLimits: { "https://foo:99999999999" : 5 }},
});

makeTest({
  name: 'perBuyerExperimentGroupIds has invalid URL in a key',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: { perBuyerExperimentGroupIds: { "https://foo:99999999999" : 11 }},
});

makeTest({
  name: 'perBuyerPrioritySignals has invalid URL in a key',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {
    perBuyerPrioritySignals: { "https://foo:99999999999" : { sig: 2.5} },
  },
});

makeTest({
  name: 'perBuyerPrioritySignals has a value with a key with prefix "browserSignals"',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {
    perBuyerPrioritySignals: { "https://example.com" : { "browserSignals.foo" : true } },
  },
});

makeTest({
  name: 'component auctions are not allowed within component auctions',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {
    interestGroupBuyers: undefined,
    componentAuctions: [
      {
        seller: window.location.origin,
        decisionLogicURL: window.location.origin,
        interestGroupBuyers: undefined,
        componentAuctions: [
          {
            seller: window.location.origin,
            decisionLogicURL: window.location.origin,
          }
        ],
      },
    ],
  },
});

makeTest({
  name: 'component auctions are not allowed with interestGroupBuyers',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {
    interestGroupBuyers: ["https://example.com"],
    componentAuctions: [
      {
        seller: window.location.origin,
        decisionLogicURL: window.location.origin,
        interestGroupBuyers: [],
      },
    ],
  },
});

makeTest({
  name: 'perBuyerCurrencies with invalid currency',
  expect: EXPECT_PROMISE_ERROR,
  expectPromiseError: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {perBuyerCurrencies: {'*': 'Dollars'}}
});

makeTest({
  name: 'perBuyerCurrencies with invalid currency map key',
  expect: EXPECT_PROMISE_ERROR,
  expectPromiseError: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {perBuyerCurrencies: {'example': 'USD'}}
});

makeTest({
  name: 'perBuyerCurrencies with non-https currency map key',
  expect: EXPECT_PROMISE_ERROR,
  expectPromiseError: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {perBuyerCurrencies: {'http://example.org/': 'USD'}}
});

makeTest({
  name: 'perBuyerCurrencies not convertible to dictionary',
  expect: EXPECT_PROMISE_ERROR,
  expectPromiseError: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {perBuyerCurrencies: 123}
});

makeTest({
  name: 'requestedSize has no width',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {requestedSize: {height: '100'}}
});

makeTest({
  name: 'requestedSize has no height',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {requestedSize: {width: '100'}}
});

makeTest({
  name: 'requestedSize width not a number',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {requestedSize: {width: '10 0', height: '100'}}
});

makeTest({
  name: 'requestedSize height not a number',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {requestedSize: {width: '100', height: '10 0'}}
});

makeTest({
  name: 'requestedSize 0',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {requestedSize: {width: '0', height: '100'}}
});

makeTest({
  name: 'requestedSize space before units',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {requestedSize: {width: '100 px', height: '100'}}
});

makeTest({
  name: 'requestedSize leading 0',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {requestedSize: {width: '0100', height: '100'}}
});

makeTest({
  name: 'requestedSize invalid unit type',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {requestedSize: {width: '100furlongs', height: '100'}}
});

makeTest({
  name: 'requestedSize hexideximal',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {requestedSize: {width: '0x100', height: '100'}}
});

makeTest({
  name: 'Empty allSlotsRequestedSizes',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {allSlotsRequestedSizes: []}
});

makeTest({
  name: 'allSlotsRequestedSizes without matching value in requestedSize',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {requestedSize:
                             {width: '100', height: '100'},
                           allSlotsRequestedSizes:
                            [{width: '100', height: '101'}]}
});

makeTest({
  name: 'allSlotsRequestedSizes has duplicate values',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {allSlotsRequestedSizes:
                            [{width: '100', height: '100'},
                             {width: '100', height: '100'}]}
});

makeTest({
  name: 'allSlotsRequestedSizes has invalid value',
  expect: EXPECT_EXCEPTION(TypeError),
  auctionConfigOverrides: {allSlotsRequestedSizes:
                            [{width: '100', height: '100'},
                             {width: '200furlongs', height: '200'}]}
});

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // The renderURL / report URLs for the first/second iterations of the auction.
  let renderURL = createRenderURL(uuid);
  let bidderReportURL1 = createBidderReportURL(uuid, /*id=*/ 1);
  let bidderReportURL2 = createBidderReportURL(uuid, /*id=*/ 2);
  let bidderDebugReportURL =
      createBidderReportURL(uuid, /*id=*/ 'forDebuggingOnly');
  let sellerReportURL1 = createSellerReportURL(uuid, /*id=*/ 1);
  let sellerReportURL2 = createSellerReportURL(uuid, /*id=*/ 2);
  let sellerDebugReportURL =
      createSellerReportURL(uuid, /*id=*/ 'forDebuggingOnly');

  // reportWin() sends "bidderReportURL1" if
  // browserSignals.forDebuggingOnlyInCooldownOrLockout is true,
  // "bidderReportURL2" otherwise.
  await joinInterestGroup(test, uuid, {
    ads: [{renderURL: renderURL}],
    biddingLogicURL: createBiddingScriptURL({
      generateBid: `
        forDebuggingOnly.reportAdAuctionWin('${bidderDebugReportURL}');
        if (!browserSignals.hasOwnProperty(
          'forDebuggingOnlyInCooldownOrLockout')) {
          throw "Missing forDebuggingOnlyInCooldownOrLockout in browserSignals";
        }
        let bid = browserSignals.forDebuggingOnlyInCooldownOrLockout ? 1 : 2;
        return {bid: bid, render: '${renderURL}'};`,
      reportWin: `
        if (browserSignals.bid === 1)
          sendReportTo('${bidderReportURL1}');
        if (browserSignals.bid === 2)
          sendReportTo('${bidderReportURL2}');`

    })
  });

  // reportResult() sends "sellerReportURL1" if
  // browserSignals.forDebuggingOnlyInCooldownOrLockout in scoreAd() is true,
  // "sellerReportURL2" otherwise.
  const auctionConfigOverrides = {
    decisionLogicURL: createDecisionScriptURL(uuid, {
      scoreAd: `
        forDebuggingOnly.reportAdAuctionWin('${sellerDebugReportURL}');
        if (!browserSignals.hasOwnProperty(
          'forDebuggingOnlyInCooldownOrLockout')) {
          throw "Missing forDebuggingOnlyInCooldownOrLockout in browserSignals";
        }
        let desirability =
            browserSignals.forDebuggingOnlyInCooldownOrLockout ? 1 : 2;
        return {desirability: desirability};`,
      reportResult: `
        if (browserSignals.desirability === 1)
          sendReportTo('${sellerReportURL1}');
        if (browserSignals.desirability === 2)
          sendReportTo('${sellerReportURL2}');`
    })
  };

  // In the first auction, browserSignals.forDebuggingOnlyInCooldownOrLockout in
  // generateBid() and scoreAd() should both be false. After the auction,
  // lockout and cooldowns should be updated.
  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfigOverrides);
  await waitForObservedRequestsIgnoreDebugOnlyReports(
      uuid, [bidderReportURL2, sellerReportURL2]);

  // In the second auction, browserSignals.forDebuggingOnlyInCooldownOrLockout
  // in generateBid() and scoreAd() should both be true, since both the buyer
  // and seller called forDebuggingOnly API in the first auction, so they are in
  // cooldowns at least (and also in lockout if a debug report is allowed to be
  // sent).
  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfigOverrides);
  await waitForObservedRequestsIgnoreDebugOnlyReports(
    uuid,
    [bidderReportURL2, sellerReportURL2, bidderReportURL1, sellerReportURL1]);
}, `forDebuggingOnly lockout and cooldowns updating in one auction, read in another's.`);

makeTest({
  name: 'deprecatedRenderURLReplacements nullability',
  expect: EXPECT_WINNER,
  auctionConfigOverrides: {deprecatedRenderURLReplacements: null}
});

makeTest({
  name: 'deprecatedRenderURLReplacements nullability 2',
  expect: EXPECT_WINNER,
  auctionConfigOverrides:
      {deprecatedRenderURLReplacements: Promise.resolve(undefined)}
});

makeTest({
  name: 'perBuyerSignals nullability',
  expect: EXPECT_WINNER,
  auctionConfigOverrides: {perBuyerSignals: null},
});

makeTest({
  name: 'perBuyerSignals nullability 2',
  expect: EXPECT_WINNER,
  auctionConfigOverrides: {perBuyerSignals: Promise.resolve(undefined)},
});

makeTest({
  name: 'perBuyerTimeouts nullability',
  expect: EXPECT_WINNER,
  auctionConfigOverrides: {perBuyerTimeouts: null},
});

makeTest({
  name: 'perBuyerTimeouts nullability 2',
  expect: EXPECT_WINNER,
  auctionConfigOverrides: {perBuyerTimeouts: Promise.resolve(undefined)},
});

makeTest({
  name: 'perBuyerCumulativeTimeouts nullability',
  expect: EXPECT_WINNER,
  auctionConfigOverrides: {perBuyerCumulativeTimeouts: null},
});

makeTest({
  name: 'perBuyerCumulativeTimeouts nullability 2',
  expect: EXPECT_WINNER,
  auctionConfigOverrides:
      {perBuyerCumulativeTimeouts: Promise.resolve(undefined)},
});

makeTest({
  name: 'perBuyerCurrencies nullability',
  expect: EXPECT_WINNER,
  auctionConfigOverrides: {perBuyerCurrencies: null},
});

makeTest({
  name: 'perBuyerCurrencies nullability 2',
  expect: EXPECT_WINNER,
  auctionConfigOverrides: {perBuyerCurrencies: Promise.resolve(undefined)},
});
