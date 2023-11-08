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
// META: variant=?26-last

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

// Expect an exception of the given type.
const EXPECT_EXCEPTION = exceptionType => auctionResult => {
  assert_not_equals(auctionResult, null, "got null instead of expected error");
  assert_true(auctionResult instanceof Error, "did not get expected error: " + auctionResult);
  assert_throws_js(exceptionType, () => { throw auctionResult; });
};

const EXPECT_PROMISE_ERROR = auctionResult => {
  assert_not_equals(auctionResult, null, "got null instead of expected error");
  // TODO(morlovich): I suspect this will end up being spec'd differently.
  assert_true(typeof auctionResult === "string",
              "did not get expected error: " + auctionResult);
}

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
  name: 'trustedScoringSignalsURL is cross-origin with seller',
  expect: EXPECT_EXCEPTION(TypeError),
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
