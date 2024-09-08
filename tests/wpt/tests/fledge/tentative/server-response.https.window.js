// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/ba-fledge-util.sub.js
// META: script=resources/fledge-util.sub.js
// META: script=third_party/cbor-js/cbor.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-4

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
    'interestGroupBuyers': [window.location.origin],
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
    'interestGroupBuyers': [window.location.origin],
    'requestId': result.requestId,
    'serverResponse': serverResponse,
    'resolveToConfig': true,
  });
  expectSuccess(auctionResult);
  createAndNavigateFencedFrame(test, auctionResult);
  await waitForObservedRequests(uuid, [adB, trackBuyer, trackSeller]);
}, 'Basic B&A auction with reporting URLs');
