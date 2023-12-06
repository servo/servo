// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-5
// META: variant=?6-10
// META: variant=?11-15
// META: variant=?16-20
// META: variant=?21-last

"use strict;"

// Subset 1 - 5
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await fetchDirectFromSellerSignals({ 'Buyer-Origin': window.location.origin });
  await runReportTest(
      test, uuid,
      directFromSellerSignalsValidatorCode(
          uuid, /*expectedSellerSignals=*/ null,
          /*expectedAuctionSignals=*/ null, /*expectedPerBuyerSignals=*/ null),
      // expectedReportUrls
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      {directFromSellerSignalsHeaderAdSlot: 'adSlot/0'});
}, 'Test directFromSellerSignals with empty Ad-Auction-Signals header.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await fetchDirectFromSellerSignals({ 'Buyer-Origin': window.location.origin });
  await runReportTest(
      test, uuid,
      directFromSellerSignalsValidatorCode(
          uuid, 'sellerSignals/1',
          /*expectedAuctionSignals=*/null, /*expectedPerBuyerSignals=*/null),
      // expectedReportUrls
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      { directFromSellerSignalsHeaderAdSlot: 'adSlot/1' }
  );
}, 'Test directFromSellerSignals with only sellerSignals.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await fetchDirectFromSellerSignals({ 'Buyer-Origin': window.location.origin });
  await runReportTest(
      test, uuid,
      directFromSellerSignalsValidatorCode(
          uuid, /*expectedSellerSignals=*/null,
          'auctionSignals/2', /*expectedPerBuyerSignals=*/null),
      // expectedReportUrls
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      { directFromSellerSignalsHeaderAdSlot: 'adSlot/2' }
  );
}, 'Test directFromSellerSignals with only auctionSignals.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await fetchDirectFromSellerSignals({ 'Buyer-Origin': window.location.origin });
  await runReportTest(
      test, uuid,
      directFromSellerSignalsValidatorCode(
          uuid, /*expectedSellerSignals=*/null,
          /*expectedAuctionSignals=*/null, 'perBuyerSignals/3'),
      // expectedReportUrls
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      { directFromSellerSignalsHeaderAdSlot: 'adSlot/3' }
  );
}, 'Test directFromSellerSignals with only perBuyerSignals.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await fetchDirectFromSellerSignals({ 'Buyer-Origin': window.location.origin });
  await runReportTest(
      test, uuid,
      directFromSellerSignalsValidatorCode(
          uuid, 'sellerSignals/4',
          'auctionSignals/4', 'perBuyerSignals/4'),
      // expectedReportUrls
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      { directFromSellerSignalsHeaderAdSlot: 'adSlot/4' }
  );
}, 'Test directFromSellerSignals with sellerSignals, auctionSignals and perBuyerSignals.');

// Subset 6 - 10
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await fetchDirectFromSellerSignals({ 'Buyer-Origin': window.location.origin });
  await runReportTest(
      test, uuid,
      directFromSellerSignalsValidatorCode(
          uuid, 'sellerSignals/1',
          /*expectedAuctionSignals=*/null, /*expectedPerBuyerSignals=*/null),
      // expectedReportUrls
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
    { directFromSellerSignalsHeaderAdSlot: 'adSlot/1' }
  );

  await runReportTest(
      test, uuid,
      directFromSellerSignalsValidatorCode(
          uuid, /*expectedSellerSignals=*/null,
          'auctionSignals/2', /*expectedPerBuyerSignals=*/null),
      // expectedReportUrls
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      { directFromSellerSignalsHeaderAdSlot: 'adSlot/2' }
  );
}, 'Test directFromSellerSignals with single fetch and multiple auctions');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const ad_slot = Promise.resolve('adSlot/4');
  await fetchDirectFromSellerSignals({ 'Buyer-Origin': window.location.origin });
  await runReportTest(
      test, uuid,
      directFromSellerSignalsValidatorCode(
          uuid, 'sellerSignals/4',
          'auctionSignals/4', 'perBuyerSignals/4'),
      // expectedReportUrls
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      { directFromSellerSignalsHeaderAdSlot: ad_slot }
  );
}, 'Test directFromSellerSignals with resolved promise ad slot.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await fetchDirectFromSellerSignals({ 'Buyer-Origin': window.location.origin });
  await joinInterestGroup(test, uuid);

  const adSlot = Promise.reject(new Error('This is a rejected promise.'));
  let auctionConfig =
      { seller: window.location.origin,
        interestGroupBuyers: [window.location.origin],
        resolveToConfig: true,
        decisionLogicURL: createDecisionScriptURL(uuid),
        directFromSellerSignalsHeaderAdSlot: adSlot };

  try {
    await navigator.runAdAuction(auctionConfig);
  } catch(e) {
    assert_true(e instanceof TypeError);
    return;
  }
  throw "Exception unexpectedly not thrown.";
}, 'Test directFromSellerSignals with rejected promise ad slot.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const validator = directFromSellerSignalsValidatorCode(
      uuid, 'sellerSignals/4',
      'auctionSignals/4', 'perBuyerSignals/4');
  let reportResult = `if (!(${validator.reportResultSuccessCondition})) {
                        sendReportTo('${createSellerReportURL(uuid, 'error')}');
                        return false;
                      }
                      ${validator.reportResult}`;
  let reportWin = `if (!(${validator.reportWinSuccessCondition})) {
                     sendReportTo('${createBidderReportURL(uuid, 'error')}');
                     return false;
                   }
                   ${validator.reportWin}`;
  let decisionScriptURLParams = { scoreAd : validator.scoreAd,
                                  reportResult : reportResult };
  let biddingScriptURLParams = { generateBid : validator.generateBid,
                                 reportWin : reportWin };
  let interestGroupOverrides =
      { biddingLogicURL: createBiddingScriptURL(biddingScriptURLParams) };
  await joinInterestGroup(test, uuid, interestGroupOverrides);

  let adSlotResolve = null;
  const adSlotPromise = new Promise((resolve, reject) => { adSlotResolve = resolve });
  let auctionConfig =
      { seller: window.location.origin,
        interestGroupBuyers: [window.location.origin],
        resolveToConfig: true,
        decisionLogicURL: createDecisionScriptURL(uuid, decisionScriptURLParams),
        directFromSellerSignalsHeaderAdSlot: adSlotPromise };
  let resultPromise = navigator.runAdAuction(auctionConfig);

  await fetchDirectFromSellerSignals({ 'Buyer-Origin': window.location.origin });
  adSlotResolve('adSlot/4');
  let result = await resultPromise;
  createAndNavigateFencedFrame(test, result);
  await waitForObservedRequests(uuid, [createSellerReportURL(uuid), createBidderReportURL(uuid)]);
}, 'Test directFromSellerSignals that runAdAuction will wait until the promise of fetch is resolved.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await fetchDirectFromSellerSignals({ 'Buyer-Origin': window.location.origin });
  await runReportTest(
      test, uuid,
      directFromSellerSignalsValidatorCode(
          uuid, 'sellerSignals/5',
          'auctionSignals/5', /*expectedPerBuyerSignals=*/null),
      // expectedReportUrls
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      { directFromSellerSignalsHeaderAdSlot: 'adSlot/5' }
  );
}, 'Test directFromSellerSignals with mismatched perBuyerSignals.');

// Subset 11 - 15
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await fetchDirectFromSellerSignals({ 'Buyer-Origin': '*' });
  await runReportTest(
      test, uuid,
      directFromSellerSignalsValidatorCode(
        uuid, 'sellerSignals/5',
        'auctionSignals/5', /*expectedPerBuyerSignals=*/null),
      // expectedReportUrls
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      { directFromSellerSignalsHeaderAdSlot: 'adSlot/5' }
  );
}, 'Test directFromSellerSignals does not support wildcard for buyerOrigin of perBuyerSignals.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await fetchDirectFromSellerSignals({ 'Buyer-Origin': window.location.origin });
  await runReportTest(
      test, uuid,
      directFromSellerSignalsValidatorCode(
          uuid, /*expectedSellerSignals=*/null,
          /*expectedAuctionSignals=*/null, /*expectedPerBuyerSignals=*/null),
      // expectedReportUrls
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      { directFromSellerSignalsHeaderAdSlot: 'adSlot/non-exist' }
  );
}, 'Test directFromSellerSignals with non-existent adSlot.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await fetchDirectFromSellerSignals({ 'Buyer-Origin': window.location.origin });
  await runReportTest(
      test, uuid,
      directFromSellerSignalsValidatorCode(
          uuid, /*expectedSellerSignals=*/null,
          /*expectedAuctionSignals=*/null, /*expectedPerBuyerSignals=*/null),
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      { directFromSellerSignalsHeaderAdSlot: null }
  );
}, 'Test directFromSellerSignals with null directFromSellerSignalsHeaderAdSlot.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await fetchDirectFromSellerSignals({ 'Buyer-Origin': window.location.origin });
  await runReportTest(
      test, uuid,
      directFromSellerSignalsValidatorCode(
          uuid, /*expectedSellerSignals=*/null,
          /*expectedAuctionSignals=*/null, /*expectedPerBuyerSignals=*/null),
      [createSellerReportURL(uuid), createBidderReportURL(uuid)]
  );
}, 'Test directFromSellerSignals with no directFromSellerSignalsHeaderAdSlot.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await fetchDirectFromSellerSignals({ 'Negative-Test-Option': 'HTTP Error' });
  await runReportTest(
      test, uuid,
      directFromSellerSignalsValidatorCode(
          uuid, /*expectedSellerSignals=*/null,
          /*expectedAuctionSignals=*/null, /*expectedPerBuyerSignals=*/null),
      // expectedReportUrls
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      { directFromSellerSignalsHeaderAdSlot: 'adSlot' }
  );
}, 'Test directFromSellerSignals with HTTP error.');

// Subset 16 - 20
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await fetchDirectFromSellerSignals({ 'Negative-Test-Option': 'No Ad-Auction-Signals Header' });
  await runReportTest(
      test, uuid,
      directFromSellerSignalsValidatorCode(
          uuid, /*expectedSellerSignals=*/null,
          /*expectedAuctionSignals=*/null, /*expectedPerBuyerSignals=*/null),
      // expectedReportUrls
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      { directFromSellerSignalsHeaderAdSlot: 'adSlot' }
  );
}, 'Test directFromSellerSignals with no returned Ad-Auction-Signals Header.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await fetchDirectFromSellerSignals({ 'Negative-Test-Option': 'Invalid Json' });
  await runReportTest(
      test, uuid,
      directFromSellerSignalsValidatorCode(
          uuid, /*expectedSellerSignals=*/null,
          /*expectedAuctionSignals=*/null, /*expectedPerBuyerSignals=*/null),
      // expectedReportUrls
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      { directFromSellerSignalsHeaderAdSlot: 'adSlot' }
  );
}, 'Test directFromSellerSignals with invalid json in Ad-Auction-Signals header.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let codeToInsert = directFromSellerSignalsValidatorCode(
      uuid, /*expectedSellerSignals=*/null,
      /*expectedAuctionSignals=*/null, /*expectedPerBuyerSignals=*/null);
  codeToInsert.decisionScriptURLOrigin = OTHER_ORIGIN1;
  await fetchDirectFromSellerSignals({ 'Buyer-Origin': window.location.origin });
  await runReportTest(
      test, uuid, codeToInsert,
      // expectedReportUrls
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      { directFromSellerSignalsHeaderAdSlot: 'adSlot/4',
        seller: OTHER_ORIGIN1 }
  );
}, 'Test directFromSellerSignals with different fetch and seller origins.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let codeToInsert = directFromSellerSignalsValidatorCode(
      uuid, 'sellerSignals/4',
      'auctionSignals/4', 'perBuyerSignals/4');
  codeToInsert.decisionScriptURLOrigin = OTHER_ORIGIN1;
  await fetchDirectFromSellerSignals({ 'Buyer-Origin': window.location.origin }, OTHER_ORIGIN1);
  await runReportTest(
      test, uuid, codeToInsert,
      // expectedReportUrls
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      { directFromSellerSignalsHeaderAdSlot: 'adSlot/4',
        seller: OTHER_ORIGIN1 }
  );
}, 'Test directFromSellerSignals with same fetch and seller origins.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let iframe = await createIframe(test, OTHER_ORIGIN1);
  await runInFrame(test, iframe, `await joinInterestGroup(test_instance, "${uuid}");`);
  await fetchDirectFromSellerSignals({ 'Buyer-Origin': OTHER_ORIGIN1 });
  await runReportTest(
      test, uuid,
      directFromSellerSignalsValidatorCode(
        uuid, 'sellerSignals/4',
        'auctionSignals/4', 'perBuyerSignals/4'),
      // expectedReportUrls
      [createSellerReportURL(uuid), createBidderReportURL(uuid, '1', OTHER_ORIGIN1)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      { directFromSellerSignalsHeaderAdSlot: 'adSlot/4',
        interestGroupBuyers: [OTHER_ORIGIN1] }
  );
}, 'Test directFromSellerSignals different interest group owner origin from top frame.');

// Subset 21 - last
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let iframe = await createIframe(test, OTHER_ORIGIN1, "join-ad-interest-group; run-ad-auction");
  await fetchDirectFromSellerSignals({ 'Buyer-Origin': OTHER_ORIGIN1 }, OTHER_ORIGIN1);
  await runInFrame(
      test, iframe,
      `await runReportTest(
          test_instance, "${uuid}",
          directFromSellerSignalsValidatorCode(
              "${uuid}", 'sellerSignals/4', 'auctionSignals/4', 'perBuyerSignals/4'),
          // expectedReportUrls
          [createSellerReportURL("${uuid}"), createBidderReportURL("${uuid}")],
          // renderURLOverride
          null,
          // auctionConfigOverrides
          { directFromSellerSignalsHeaderAdSlot: 'adSlot/4' })`);
}, 'Test directFromSellerSignals with fetching in top frame and running auction in iframe.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let iframe = await createIframe(test, OTHER_ORIGIN1, "join-ad-interest-group; run-ad-auction");
  await runInFrame(
      test, iframe,
      `await fetchDirectFromSellerSignals({ 'Buyer-Origin': window.location.origin });
       await runReportTest(
          test_instance, "${uuid}",
          directFromSellerSignalsValidatorCode(
              "${uuid}", 'sellerSignals/4',
              'auctionSignals/4', 'perBuyerSignals/4'),
          // expectedReportUrls
          [createSellerReportURL("${uuid}"), createBidderReportURL("${uuid}")],
          // renderURLOverride
          null,
          // auctionConfigOverrides
          { directFromSellerSignalsHeaderAdSlot: 'adSlot/4' })`);
}, 'Test directFromSellerSignals with fetching and running auction in the same iframe.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let iframe1 = await createIframe(test, OTHER_ORIGIN1);
  let iframe2 = await createIframe(test, OTHER_ORIGIN2, "join-ad-interest-group; run-ad-auction");
  await runInFrame(
      test, iframe1,
      `await fetchDirectFromSellerSignals({ 'Buyer-Origin': OTHER_ORIGIN2 }, OTHER_ORIGIN2);`);
  await runInFrame(
      test, iframe2,
      `await runReportTest(
          test_instance, "${uuid}",
          directFromSellerSignalsValidatorCode(
              "${uuid}", 'sellerSignals/4',
              'auctionSignals/4', 'perBuyerSignals/4'),
          // expectedReportUrls
          [createSellerReportURL("${uuid}"), createBidderReportURL("${uuid}")],
          // renderURLOverride
          null,
          // auctionConfigOverrides
          { directFromSellerSignalsHeaderAdSlot: 'adSlot/4' })`);
}, 'Test directFromSellerSignals with fetching in iframe 1 and running auction in iframe 2.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let iframe = await createIframe(test, OTHER_ORIGIN1);
  await runInFrame(
      test, iframe,
      `await fetchDirectFromSellerSignals(
          { 'Buyer-Origin': "${window.location.origin}" }, "${window.location.origin}");`);
  await runReportTest(
      test, uuid,
      directFromSellerSignalsValidatorCode(
          uuid, 'sellerSignals/4',
          'auctionSignals/4', 'perBuyerSignals/4'),
      // expectedReportUrls
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      { directFromSellerSignalsHeaderAdSlot: 'adSlot/4'}
  );
}, 'Test directFromSellerSignals with fetching in iframe and running auction in top frame.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await fetchDirectFromSellerSignals({ 'Negative-Test-Option': 'Network Error' });
  await runReportTest(
      test, uuid,
      directFromSellerSignalsValidatorCode(
          uuid, 'sellerSignals',
          'auctionSignals', /*expectedPerBuyerSignals=*/null),
      // expectedReportUrls
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      { directFromSellerSignalsHeaderAdSlot: 'adSlot' }
  );
}, 'Test directFromSellerSignals with network error.');

subsetTest(promise_test, async test => {
  let dfss = false;
  navigator.runAdAuction({
      get directFromSellerSignalsHeaderAdSlot() { dfss = true; }
  }).catch((e) => {});
  assert_true(dfss);
}, 'Test directFromSellerSignals feature detection.');
