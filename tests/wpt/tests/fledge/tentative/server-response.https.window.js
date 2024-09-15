// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/ba-fledge-util.sub.js
// META: script=resources/fledge-util.sub.js
// META: script=third_party/cbor-js/cbor.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-4
// META: variant=?5-8
// META: variant=?9-12

// These tests focus on the serverResponse field in AuctinConfig, e.g.
// auctions involving bidding and auction services.

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const adA = createTrackerURL(window.location.origin, uuid, 'track_get', 'a');
  const adB = createTrackerURL(window.location.origin, uuid, 'track_get', 'b');
  const adsArray =
      [{renderURL: adA, adRenderId: 'a'}, {renderURL: adB, adRenderId: 'b'}];
  await joinInterestGroup(test, uuid, {ads: adsArray});

  const result = await navigator.getInterestGroupAdAuctionData(
      {seller: window.location.origin});
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);

  let serverResponseMsg = {
    'biddingGroups': {},
    'adRenderURL': adsArray[0].renderURL,
    'interestGroupName': DEFAULT_INTEREST_GROUP_NAME,
    'interestGroupOwner': window.location.origin,
  };
  serverResponseMsg.biddingGroups[window.location.origin] = [0];

  let serverResponse =
      await BA.encodeServerResponse(serverResponseMsg, decoded);

  let hashString = await BA.payloadHash(serverResponse);
  await BA.authorizeServerResponseHashes([hashString]);

  let auctionResult = await navigator.runAdAuction({
    'seller': window.location.origin,
    'requestId': result.requestId,
    'serverResponse': serverResponse,
    'resolveToConfig': true,
  });
  expectSuccess(auctionResult);
  createAndNavigateFencedFrame(test, auctionResult);
  await waitForObservedRequests(uuid, [adA]);
}, 'Basic B&A auction');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const adA = createTrackerURL(window.location.origin, uuid, 'track_get', 'a');
  const adB = createTrackerURL(window.location.origin, uuid, 'track_get', 'b');
  const adsArray =
      [{renderURL: adA, adRenderId: 'a'}, {renderURL: adB, adRenderId: 'b'}];
  await joinInterestGroup(test, uuid, {ads: adsArray});

  const result = await navigator.getInterestGroupAdAuctionData(
      {seller: window.location.origin});
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);

  const trackSeller = createSellerReportURL(uuid);
  const trackBuyer = createBidderReportURL(uuid);
  let serverResponseMsg = {
    'biddingGroups': {},
    'adRenderURL': adsArray[1].renderURL,
    'interestGroupName': DEFAULT_INTEREST_GROUP_NAME,
    'interestGroupOwner': window.location.origin,
    'winReportingURLs': {
      'buyerReportingURLs': {'reportingURL': trackBuyer},
      'topLevelSellerReportingURLs': {'reportingURL': trackSeller}
    }
  };
  serverResponseMsg.biddingGroups[window.location.origin] = [0];

  let serverResponse =
      await BA.encodeServerResponse(serverResponseMsg, decoded);

  let hashString = await BA.payloadHash(serverResponse);
  await BA.authorizeServerResponseHashes([hashString]);

  let auctionResult = await navigator.runAdAuction({
    'seller': window.location.origin,
    'requestId': result.requestId,
    'serverResponse': serverResponse,
    'resolveToConfig': true,
  });
  expectSuccess(auctionResult);
  createAndNavigateFencedFrame(test, auctionResult);
  await waitForObservedRequests(uuid, [adB, trackBuyer, trackSeller]);
}, 'Basic B&A auction with reporting URLs');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const adA = createTrackerURL(window.location.origin, uuid, 'track_get', 'a');
  const adB = createTrackerURL(window.location.origin, uuid, 'track_get', 'b');
  const adsArray =
      [{renderURL: adA, adRenderId: 'a'}, {renderURL: adB, adRenderId: 'b'}];
  await joinInterestGroup(test, uuid, {
    ads: adsArray,
    biddingLogicURL: createBiddingScriptURL({allowComponentAuction: true})
  });

  const result = await navigator.getInterestGroupAdAuctionData(
      {seller: window.location.origin});
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);

  // The server-side auction uses a bid of 10, for second ad, so it should
  // win over the client-side component auctions bid of 9.
  let serverResponseMsg = {
    'biddingGroups': {},
    'adRenderURL': adsArray[1].renderURL,
    'interestGroupName': DEFAULT_INTEREST_GROUP_NAME,
    'interestGroupOwner': window.location.origin,
    'topLevelSeller': window.location.origin,
    'bid': 10,
  };
  serverResponseMsg.biddingGroups[window.location.origin] = [0];

  let serverResponse =
      await BA.encodeServerResponse(serverResponseMsg, decoded);

  let hashString = await BA.payloadHash(serverResponse);
  await BA.authorizeServerResponseHashes([hashString]);

  let auctionConfig = {
    seller: window.location.origin,
    decisionLogicURL: createDecisionScriptURL(uuid),
    interestGroupBuyers: [],
    resolveToConfig: true,
    componentAuctions: [
      {
        seller: window.location.origin,
        decisionLogicURL: createDecisionScriptURL(uuid),
        interestGroupBuyers: [window.location.origin],
      },
      {
        seller: window.location.origin,
        requestId: result.requestId,
        serverResponse: serverResponse,
      }
    ]
  };

  let auctionResult = await navigator.runAdAuction(auctionConfig);
  expectSuccess(auctionResult);
  createAndNavigateFencedFrame(test, auctionResult);
  await waitForObservedRequests(uuid, [adB]);
}, 'Hybrid B&A auction');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const adA = createTrackerURL(window.location.origin, uuid, 'track_get', 'a');
  const adB = createTrackerURL(window.location.origin, uuid, 'track_get', 'b');
  const adsArray =
      [{renderURL: adA, adRenderId: 'a'}, {renderURL: adB, adRenderId: 'b'}];
  await joinInterestGroup(test, uuid, {
    ads: adsArray,
    biddingLogicURL: createBiddingScriptURL({allowComponentAuction: true})
  });

  const result = await navigator.getInterestGroupAdAuctionData(
      {seller: window.location.origin});
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);

  // The server-side auction uses a bid of 10, for second ad, so it should
  // win over the client-side component auctions bid of 9.
  const trackServerSeller = createSellerReportURL(uuid);
  const trackBuyer = createBidderReportURL(uuid);
  let serverResponseMsg = {
    'biddingGroups': {},
    'adRenderURL': adsArray[1].renderURL,
    'interestGroupName': DEFAULT_INTEREST_GROUP_NAME,
    'interestGroupOwner': window.location.origin,
    'topLevelSeller': window.location.origin,
    'bid': 10,
    'winReportingURLs': {
      'buyerReportingURLs': {'reportingURL': trackBuyer},
      'componentSellerReportingURLs': {'reportingURL': trackServerSeller}
    }
  };
  serverResponseMsg.biddingGroups[window.location.origin] = [0];

  let serverResponse =
      await BA.encodeServerResponse(serverResponseMsg, decoded);

  let hashString = await BA.payloadHash(serverResponse);
  await BA.authorizeServerResponseHashes([hashString]);

  let trackTopSeller = createSellerReportURL(uuid, 'top');
  let trackClientSeller = createSellerReportURL(uuid, 'client');
  let auctionConfig = {
    seller: window.location.origin,
    decisionLogicURL: createDecisionScriptURL(
        uuid, {reportResult: `sendReportTo("${trackTopSeller}")`}),
    interestGroupBuyers: [],
    resolveToConfig: true,
    componentAuctions: [
      {
        seller: window.location.origin,
        decisionLogicURL: createDecisionScriptURL(
            uuid, {reportResult: `sendReportTo("${trackClientSeller}")`}),
        interestGroupBuyers: [window.location.origin],
      },
      {
        seller: window.location.origin,
        requestId: result.requestId,
        serverResponse: serverResponse,
      }
    ]
  };

  let auctionResult = await navigator.runAdAuction(auctionConfig);
  expectSuccess(auctionResult);
  createAndNavigateFencedFrame(test, auctionResult);
  await waitForObservedRequests(
      uuid, [adB, trackBuyer, trackServerSeller, trackTopSeller]);
}, 'Hybrid B&A auction with reporting URLs');

async function runFaultInjectTest(test, fault) {
  const uuid = generateUuid(test);
  const adA = 'https://example.org/a';
  const adB = 'https://example.org/b';
  const adsArray =
      [{renderURL: adA, adRenderId: 'a'}, {renderURL: adB, adRenderId: 'b'}];
  await joinInterestGroup(test, uuid, {ads: adsArray});

  const result = await navigator.getInterestGroupAdAuctionData(
      {seller: window.location.origin});
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);

  let serverResponseMsg = {
    'biddingGroups': {},
    'adRenderURL': adsArray[0].renderURL,
    'interestGroupName': DEFAULT_INTEREST_GROUP_NAME,
    'interestGroupOwner': window.location.origin,
  };
  serverResponseMsg.biddingGroups[window.location.origin] = [0];

  let serverResponse =
      await BA.encodeServerResponse(serverResponseMsg, decoded, fault);

  let hashString = await BA.payloadHash(serverResponse);
  await BA.authorizeServerResponseHashes([hashString]);

  let auctionResult = await navigator.runAdAuction({
    'seller': window.location.origin,
    'requestId': result.requestId,
    'serverResponse': serverResponse,
    'resolveToConfig': true,
  });
  expectNoWinner(auctionResult);
}

subsetTest(promise_test, async test => {
  return runFaultInjectTest(test, BA.injectCborFault);
}, 'Basic B&A auction - fault inject at CBOR');

subsetTest(promise_test, async test => {
  return runFaultInjectTest(test, BA.injectGzipFault);
}, 'Basic B&A auction - fault inject at gzip');

subsetTest(promise_test, async test => {
  return runFaultInjectTest(test, BA.injectFrameFault);
}, 'Basic B&A auction - fault inject at framing');

subsetTest(promise_test, async test => {
  return runFaultInjectTest(test, BA.injectEncryptFault);
}, 'Basic B&A auction - fault inject at encryption');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const adA = 'https://example.org/a';
  const adB = 'https://example.org/b';
  const adsArray =
      [{renderURL: adA, adRenderId: 'a'}, {renderURL: adB, adRenderId: 'b'}];
  await joinInterestGroup(test, uuid, {ads: adsArray});

  const result = await navigator.getInterestGroupAdAuctionData(
      {seller: window.location.origin});
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);

  let serverResponseMsg = {
    'biddingGroups': {},
    'adRenderURL': adsArray[0].renderURL,
    'interestGroupName': DEFAULT_INTEREST_GROUP_NAME,
    'interestGroupOwner': window.location.origin,
  };
  serverResponseMsg.biddingGroups[window.location.origin] = [0];

  let serverResponse =
      await BA.encodeServerResponse(serverResponseMsg, decoded);

  // Mess up the array for a bit before computing hash to get the wrong hash,
  // then undo.
  serverResponse[0] ^= 0xBE;
  let hashString = await BA.payloadHash(serverResponse);
  await BA.authorizeServerResponseHashes([hashString]);
  serverResponse[0] ^= 0xBE;

  let auctionResult = await navigator.runAdAuction({
    'seller': window.location.origin,
    'requestId': result.requestId,
    'serverResponse': serverResponse,
    'resolveToConfig': true,
  });
  expectNoWinner(auctionResult);
}, 'Basic B&A auction - Wrong authorization');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const adA = 'https://example.org/a';
  const adB = 'https://example.org/b';
  const adsArray =
      [{renderURL: adA, adRenderId: 'a'}, {renderURL: adB, adRenderId: 'b'}];
  await joinInterestGroup(test, uuid, {ads: adsArray});

  const result =
      await navigator.getInterestGroupAdAuctionData({seller: OTHER_ORIGIN1});
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);

  let serverResponseMsg = {
    'biddingGroups': {},
    'adRenderURL': adsArray[0].renderURL,
    'interestGroupName': DEFAULT_INTEREST_GROUP_NAME,
    'interestGroupOwner': window.location.origin,
  };
  serverResponseMsg.biddingGroups[window.location.origin] = [0];

  let serverResponse =
      await BA.encodeServerResponse(serverResponseMsg, decoded);

  let hashString = await BA.payloadHash(serverResponse);
  await BA.authorizeServerResponseHashes([hashString]);

  let auctionResult = await navigator.runAdAuction({
    'seller': window.location.origin,
    'requestId': result.requestId,
    'serverResponse': serverResponse,
    'resolveToConfig': true,
  });
  expectNoWinner(auctionResult);
}, 'Basic B&A auction - Wrong seller');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const adA = 'https://example.org/a';
  const adB = 'https://example.org/b';
  const adsArray =
      [{renderURL: adA, adRenderId: 'a'}, {renderURL: adB, adRenderId: 'b'}];
  await joinInterestGroup(test, uuid, {ads: adsArray});

  const result = await navigator.getInterestGroupAdAuctionData(
      {seller: window.location.origin});
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);

  let serverResponseMsg = {
    'biddingGroups': {},
    'adRenderURL': adsArray[0].renderURL,
    'interestGroupName': DEFAULT_INTEREST_GROUP_NAME,
    'interestGroupOwner': window.location.origin,
  };
  serverResponseMsg.biddingGroups[window.location.origin] = [0];

  let serverResponse =
      await BA.encodeServerResponse(serverResponseMsg, decoded);

  let hashString = await BA.payloadHash(serverResponse);
  await BA.authorizeServerResponseHashes([hashString]);

  let auctionResult = await navigator.runAdAuction({
    'seller': window.location.origin,
    'requestId': token(),
    'serverResponse': serverResponse,
    'resolveToConfig': true,
  });
  expectNoWinner(auctionResult);
}, 'Basic B&A auction - Wrong request Id');
