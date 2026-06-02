// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-last

"use strict";

// This file contains tests for additional bids and negative targeting.
//
// TODO:
// - test that an additional bid with some correct signatures can be negative
//       targeted for those negative interest groups whose signatures match.
// - test that additional bids can be fetched using an iframe navigation.
// - test that additional bids are not fetched using an iframe navigation for
//      which the `adAuctionHeaders=true` attribute is not specified.
// - test that additional bids are not fetched using a Fetch request for which
//      `adAuctionHeaders: true` is not specified.
// - test that an additional bid with an incorrect seller and / or top-level
//       seller is not included in an auction.
// - lots of tests for different types of malformed additional bids, e.g.
//       missing fields, malformed signature, invalid currency code,
//       missing joining origin for multiple negative interest groups, etc.
// - test that correctly formatted additional bids are included in an auction
//       when fetched alongside malformed additional bid headers by a Fetch
//       request (both invalid headers and invalid additional bids)
// - test that an additional bid is rejected if its from a buyer who is not
//       allowed to participate in the auction.
// - test that an additional bid is rejected if its currency doesn't match the
//       buyer's associated per-buyer currency from the auction config.
// - test that correctly formatted additional bids are included in an auction
//       when fetched alongside malformed additional bid headers by an iframe
//       navigation (both invalid headers and invalid additional bids).
// - test that reportWin is not used for reporting an additional bid win.
// - test that additional bids can *not* be fetched from iframe subresource
//       requests.
// - test that an auction nonce can only be used once, and a second auction
//       trying to reuse an auction immediately fails.
// - test that an auction nonce must be created in the same window/tab as the
//       call to runAdAuction.
// - test reportAdditionalBidWin with each of no metadata, null metadata, and
//       an object metadata.
// - test that an auction running in one tab can't see an additional bid loaded
//       in a new tab.
// - test that two auctions running with different nonces only get the
//       additional bids fetched with their auction nonce.
// - test that two auctions running with different nonces only get the
//       additional bids fetched with their auction nonce, when both additional
//       bids are retrieved with one fetch.
// - test that a multiseller auction with two component auctions can direct
//       additional bids to the correct component auctions.
// - test that two auctions running with different nonces only get the
//       additional bids fetched with their auction nonce.
// - test that two auctions running with different nonces only get the
//       additional bids fetched with their auction nonce, when both additional
//       bids are retrieved with one fetch.
// - test that an additional bid can compete against an interest group bid and
//       lose.
// - test that an additional bid can compete against an interest group bid and
//       win.
// - test (in join-leave-ad-interest-group.https.window.js) that an IG that
//       provides `additionalBidKey` fails if the key fails to decode, or if
//       that IG also provides `ads`, or if it provides `updateURL`.
// - test that an IG update cannot cause a regular interest group (one that
//       does not provide `additionalBidKey`) to become a negative interest
//       group (one that does provide `additionalBidKey`).
// - test (in auction-config-passed-to-worklets.https.window.js) that a
//       multi-seller auction fails if the top-level auction provides
//       a value for `additionalBids`.
// - test (in auction-config-passed-to-worklets.https.window.js) that an auction
//       fails if it provides `additionalBids` but not `auctionNonce`, or if it
//       provides `additionalBids` but not `interestGroupBuyers`.

// The auction is run with the seller being the same as the document origin.
// The request to fetch additional bids must be issued to the seller's origin
// for ad auction headers interception to associate it with this auction.
const SINGLE_SELLER_AUCTION_SELLER = window.location.origin;

const ADDITIONAL_BID_SECRET_KEY = 'nWGxne/9WmC6hEr0kuwsxERJxWl7MmkZcDusAxyuf2A=';
const ADDITIONAL_BID_PUBLIC_KEY = '11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo=';

// Single-seller auction with a single buyer who places a single additional
// bid. As the only bid, this wins.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const buyer = OTHER_ORIGIN1;
  const additionalBid = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer, 'horses', 1.99);
  additionalBid.auctionNonce = auctionNonce;
  additionalBidHelper.setAuctionNonceInHeader(additionalBid, auctionNonce);

  await runAdditionalBidTest(
      test, uuid, [buyer], auctionNonce,
      additionalBidHelper.fetchAdditionalBids(seller, [additionalBid]),
      /*highestScoringOtherBid=*/0,
      /*winningAdditionalBidId=*/'horses');
}, 'single valid additional bid');

// Single-seller auction with a two buyers competing with additional bids.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const buyer1 = OTHER_ORIGIN1;
  const additionalBid1 = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer1, 'horses', 1.99);
  additionalBid1.auctionNonce = auctionNonce;
  additionalBidHelper.setAuctionNonceInHeader(additionalBid1, auctionNonce);

  const buyer2 = OTHER_ORIGIN2;
  const additionalBid2 = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer2, 'planes', 2.99);
  additionalBid2.auctionNonce = auctionNonce;
  additionalBidHelper.setAuctionNonceInHeader(additionalBid2, auctionNonce);

  await runAdditionalBidTest(
      test, uuid, [buyer1, buyer2], auctionNonce,
      additionalBidHelper.fetchAdditionalBids(
          seller, [additionalBid1, additionalBid2]),
      /*highestScoringOtherBid=*/1.99,
      /*winningAdditionalBidId=*/'planes');
}, 'two valid additional bids');

// Same as the test above, except that this uses two Fetch requests instead of
// one to retrieve the additional bids.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const buyer1 = OTHER_ORIGIN1;
  const additionalBid1 = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer1, 'horses', 1.99);
  additionalBid1.auctionNonce = auctionNonce;
  additionalBidHelper.setAuctionNonceInHeader(additionalBid1, auctionNonce);

  const buyer2 = OTHER_ORIGIN2;
  const additionalBid2 = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer2, 'planes', 2.99);
  additionalBid2.auctionNonce = auctionNonce;
  additionalBidHelper.setAuctionNonceInHeader(additionalBid2, auctionNonce);

  await runAdditionalBidTest(
    test, uuid, [buyer1, buyer2], auctionNonce,
    Promise.all([
        additionalBidHelper.fetchAdditionalBids(seller, [additionalBid1]),
        additionalBidHelper.fetchAdditionalBids(seller, [additionalBid2])
    ]),
    /*highestScoringOtherBid=*/1.99,
    /*winningAdditionalBidId=*/'planes');
}, 'two valid additional bids from two distinct Fetch requests');

// Single-seller auction with a single buyer who places a single additional
// bid with the wrong `auctionNonce` in the bid, causing the bid to fail.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonceInHeader = await navigator.createAuctionNonce();
  const auctionNonceInBid = crypto.randomUUID();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const buyer = OTHER_ORIGIN1;
  const additionalBid = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer, 'horses', 1.99);
  additionalBid.additionalBid = auctionNonceInBid;
  additionalBidHelper.setAuctionNonceInHeader(additionalBid, auctionNonceInHeader);

  await runAdditionalBidTestNoWinner(
      test, uuid, [buyer], auctionNonceInHeader,
      additionalBidHelper.fetchAdditionalBids(seller, [additionalBid]));
}, 'single valid additional bid with wrong auctionNonce in bid');

// Single-seller auction with a single buyer who places a single additional
// bid with no `auctionNonce` in the bid, causing the bid to fail.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const buyer = OTHER_ORIGIN1;
  const additionalBid = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer, 'horses', 1.99);
  // Notably missing: `additionalBid.auctionNonce`
  additionalBidHelper.setAuctionNonceInHeader(additionalBid, auctionNonce);

  await runAdditionalBidTestNoWinner(
      test, uuid, [buyer], auctionNonce,
      additionalBidHelper.fetchAdditionalBids(seller, [additionalBid]));
}, 'single valid additional bid with no auctionNonce in bid');

// Single-seller auction with a single buyer who places a single additional
// bid that elides `auctionNonce` in favor of a `bidNonce`, which is correctly
// computed from the combination of `auctionNonce` and `sellerNonce`. However,
// this test fails to include the `sellerNonce` in the header,
// causing the bid to fail. `bidNonce` in an additional bid is only valid with
// a `sellerNonce` on the response header.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const sellerNonce = crypto.randomUUID();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const buyer = OTHER_ORIGIN1;
  const additionalBid = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer, 'horses', 1.99);
  additionalBid.bidNonce =
      await additionalBidHelper.computeBidNonce(auctionNonce, sellerNonce);
  additionalBidHelper.setAuctionNonceInHeader(additionalBid, auctionNonce);
  // Notably missing: `additionalBidHelper.setSellerNonceInHeader`

  await runAdditionalBidTestNoWinner(
      test, uuid, [buyer], auctionNonce,
      additionalBidHelper.fetchAdditionalBids(seller, [additionalBid]));
}, 'single valid additional bid with bidNonce in bid but no sellerNonce in header');

// Single-seller auction with a single buyer who places a single additional
// bid that uses `sellerNonce` / `bidNonce`. As the only bid, this wins.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const sellerNonce = crypto.randomUUID();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const buyer = OTHER_ORIGIN1;
  const additionalBid = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer, 'horses', 1.99);
  additionalBid.bidNonce =
      await additionalBidHelper.computeBidNonce(auctionNonce, sellerNonce);
  additionalBidHelper.setAuctionNonceInHeader(additionalBid, auctionNonce);
  additionalBidHelper.setSellerNonceInHeader(additionalBid, sellerNonce);

  await runAdditionalBidTest(
      test, uuid, [buyer], auctionNonce,
      additionalBidHelper.fetchAdditionalBids(seller, [additionalBid]),
      /*highestScoringOtherBid=*/0,
      /*winningAdditionalBidId=*/'horses');
}, 'single valid additional bid with bidNonce and sellerNonce');

// Single-seller auction with a single buyer who places a single additional bid
// that uses `sellerNonce`, but with an `auctionNonce` on the bid instead of a
// `bidNonce`. Since `sellerNonce` is only compatible with `bidNonce`, there is
// no winner.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const sellerNonce = crypto.randomUUID();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const buyer = OTHER_ORIGIN1;
  const additionalBid = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer, 'horses', 1.99);
  additionalBid.auctionNonce = auctionNonce;
  additionalBidHelper.setAuctionNonceInHeader(additionalBid, auctionNonce);
  additionalBidHelper.setSellerNonceInHeader(additionalBid, sellerNonce);

  await runAdditionalBidTestNoWinner(
      test, uuid, [buyer], auctionNonce,
      additionalBidHelper.fetchAdditionalBids(seller, [additionalBid]));
}, 'single additional bid with sellerNonce in the header but auctionNonce in the bid');

// Single-seller auction with a single buyer who places a single additional bid
// that uses `sellerNonce`, but no `bidNonce` or `auctionNonce`. Since the
// `bidNonce` is missing, there is no winner. Related to 'single valid
// additional bid with no auctionNonce in bid', except with `sellerNonce`.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const sellerNonce = crypto.randomUUID();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const buyer = OTHER_ORIGIN1;
  const additionalBid = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer, 'horses', 1.99);
  // Notably missing: `additionalBid.bidNonce`.
  additionalBidHelper.setAuctionNonceInHeader(additionalBid, auctionNonce);
  additionalBidHelper.setSellerNonceInHeader(additionalBid, sellerNonce);

  await runAdditionalBidTestNoWinner(
      test, uuid, [buyer], auctionNonce,
      additionalBidHelper.fetchAdditionalBids(seller, [additionalBid]));
}, 'single additional bid with sellerNonce in the header but no bidNonce or ' +
   'auctionNonce in the bid');

// Single-seller auction with a single buyer who places a single additional
// bid that uses `sellerNonce` / `bidNonce`, but also `auctionNonce` in the bid.
// As exactly one of `bidNonce` / `auctionNonce` is allowed in the bid, this
// fails.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const sellerNonce = crypto.randomUUID();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const buyer = OTHER_ORIGIN1;
  const additionalBid = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer, 'horses', 1.99);
  additionalBid.auctionNonce = auctionNonce;
  additionalBid.bidNonce =
      await additionalBidHelper.computeBidNonce(auctionNonce, sellerNonce);
  additionalBidHelper.setAuctionNonceInHeader(additionalBid, auctionNonce);
  additionalBidHelper.setSellerNonceInHeader(additionalBid, sellerNonce);

  await runAdditionalBidTestNoWinner(
      test, uuid, [buyer], auctionNonce,
      additionalBidHelper.fetchAdditionalBids(seller, [additionalBid]));
}, 'single additional bid with sellerNonce in the header but with both ' +
   ' bidNonce and auctionNonce in the bid');

// Single-seller auction with a single buyer who places a single additional
// bid that uses `sellerNonce` / `bidNonce`. Since the `bidNonce` is invalid
// there is no winner.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const sellerNonce = crypto.randomUUID();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const buyer = OTHER_ORIGIN1;
  const additionalBid = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer, 'horses', 1.99);
  // Computes a `bidNonce` using a randomly generated `sellerNonce`, not the
  // one used in the header.
  additionalBid.bidNonce = await additionalBidHelper.computeBidNonce(
      auctionNonce, crypto.randomUUID());
  additionalBidHelper.setAuctionNonceInHeader(additionalBid, auctionNonce);
  additionalBidHelper.setSellerNonceInHeader(additionalBid, sellerNonce);

  await runAdditionalBidTestNoWinner(
      test, uuid, [buyer], auctionNonce,
      additionalBidHelper.fetchAdditionalBids(seller, [additionalBid]));
}, 'single additional bid with invalid bidNonce');

// Single-seller auction with a two buyers competing with additional bids, each
// using a distinct `sellerNonce` / `bidNonce`.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const sellerNonce1 = crypto.randomUUID();
  const sellerNonce2 = crypto.randomUUID();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const buyer1 = OTHER_ORIGIN1;
  const additionalBid1 = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer1, 'horses', 1.99);
  additionalBid1.bidNonce =
      await additionalBidHelper.computeBidNonce(auctionNonce, sellerNonce1);
  additionalBidHelper.setAuctionNonceInHeader(additionalBid1, auctionNonce);
  additionalBidHelper.setSellerNonceInHeader(additionalBid1, sellerNonce1);

  const buyer2 = OTHER_ORIGIN2;
  const additionalBid2 = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer2, 'planes', 2.99);
  additionalBid2.bidNonce =
      await additionalBidHelper.computeBidNonce(auctionNonce, sellerNonce2);
  additionalBidHelper.setAuctionNonceInHeader(additionalBid2, auctionNonce);
  additionalBidHelper.setSellerNonceInHeader(additionalBid2, sellerNonce2);

  await runAdditionalBidTest(
      test, uuid, [buyer1, buyer2], auctionNonce,
      additionalBidHelper.fetchAdditionalBids(
          seller, [additionalBid1, additionalBid2]),
      /*highestScoringOtherBid=*/1.99,
      /*winningAdditionalBidId=*/'planes');
}, 'two valid additional bids using distinct bidNonces and sellerNonces');

// Single-seller auction with a single additional bid. Because this additional
// bid is filtered by negative targeting, this auction has no winner.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const negativeInterestGroupName = 'already-owns-a-plane';

  const buyer = OTHER_ORIGIN1;
  const additionalBid = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer, 'planes', 2.99);
  additionalBid.auctionNonce = auctionNonce;
  additionalBidHelper.setAuctionNonceInHeader(additionalBid, auctionNonce);
  additionalBidHelper.addNegativeInterestGroup(
      additionalBid, negativeInterestGroupName);
  additionalBidHelper.signWithSecretKeys(
      additionalBid, [ADDITIONAL_BID_SECRET_KEY]);

  await joinNegativeInterestGroup(
      test, buyer, negativeInterestGroupName, ADDITIONAL_BID_PUBLIC_KEY);

  await runBasicFledgeTestExpectingNoWinner(
      test, uuid,
      { interestGroupBuyers: [buyer],
        auctionNonce: auctionNonce,
        additionalBids: additionalBidHelper.fetchAdditionalBids(
            seller, [additionalBid])});
}, 'one additional bid filtered by negative targeting, so auction has no ' +
   'winner');

// Single-seller auction with a two buyers competing with additional bids.
// The higher of these has a negative interest group specified, and that
// negative interest group has been joined, so the lower bid wins.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const negativeInterestGroupName = 'already-owns-a-plane';

  const buyer1 = OTHER_ORIGIN1;
  const additionalBid1 = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer1, 'horses', 1.99);
  additionalBid1.auctionNonce = auctionNonce;
  additionalBidHelper.setAuctionNonceInHeader(additionalBid1, auctionNonce);

  const buyer2 = OTHER_ORIGIN2;
  const additionalBid2 = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer2, 'planes', 2.99);
  additionalBid2.auctionNonce = auctionNonce;
  additionalBidHelper.setAuctionNonceInHeader(additionalBid2, auctionNonce);
  additionalBidHelper.addNegativeInterestGroup(
      additionalBid2, negativeInterestGroupName);
  additionalBidHelper.signWithSecretKeys(
      additionalBid2, [ADDITIONAL_BID_SECRET_KEY]);

  await joinNegativeInterestGroup(
      test, buyer2, negativeInterestGroupName, ADDITIONAL_BID_PUBLIC_KEY);

  await runAdditionalBidTest(
    test, uuid, [buyer1, buyer2], auctionNonce,
    additionalBidHelper.fetchAdditionalBids(
        seller, [additionalBid1, additionalBid2]),
    /*highestScoringOtherBid=*/0,
    /*winningAdditionalBidId=*/'horses');
}, 'higher additional bid is filtered by negative targeting, so ' +
   'lower additional bid wins');

// Same as above, except that the bid is missing a signature, so that the
// negative targeting interest group is ignored, and the higher bid, which
// would have otherwise been filtered by negative targeting, wins.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const negativeInterestGroupName = 'already-owns-a-plane';

  const buyer1 = OTHER_ORIGIN1;
  const additionalBid1 = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer1, 'horses', 1.99);
  additionalBid1.auctionNonce = auctionNonce;
  additionalBidHelper.setAuctionNonceInHeader(additionalBid1, auctionNonce);

  const buyer2 = OTHER_ORIGIN2;
  const additionalBid2 = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer2, 'planes', 2.99);
  additionalBid2.auctionNonce = auctionNonce;
  additionalBidHelper.setAuctionNonceInHeader(additionalBid2, auctionNonce);
  additionalBidHelper.addNegativeInterestGroup(
      additionalBid2, negativeInterestGroupName);

  await joinNegativeInterestGroup(
      test, buyer2, negativeInterestGroupName, ADDITIONAL_BID_PUBLIC_KEY);

  await runAdditionalBidTest(
    test, uuid, [buyer1, buyer2], auctionNonce,
    additionalBidHelper.fetchAdditionalBids(
        seller, [additionalBid1, additionalBid2]),
    /*highestScoringOtherBid=*/1.99,
    /*winningAdditionalBidId=*/'planes');
}, 'higher additional bid is filtered by negative targeting, but it is ' +
   'missing a signature, so it still wins');

// Same as above, except that the bid is signed incorrectly, so that the
// negative targeting interest group is ignored, and the higher bid, which
// would have otherwise been filtered by negative targeting, wins.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const negativeInterestGroupName = 'already-owns-a-plane';

  const buyer1 = OTHER_ORIGIN1;
  const additionalBid1 = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer1, 'horses', 1.99);
  additionalBid1.auctionNonce = auctionNonce;
  additionalBidHelper.setAuctionNonceInHeader(additionalBid1, auctionNonce);

  const buyer2 = OTHER_ORIGIN2;
  const additionalBid2 = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer2, 'planes', 2.99);
  additionalBid2.auctionNonce = auctionNonce;
  additionalBidHelper.setAuctionNonceInHeader(additionalBid2, auctionNonce);
  additionalBidHelper.addNegativeInterestGroup(
      additionalBid2, negativeInterestGroupName);
  additionalBidHelper.incorrectlySignWithSecretKeys(
      additionalBid2, [ADDITIONAL_BID_SECRET_KEY]);

  await joinNegativeInterestGroup(
      test, buyer2, negativeInterestGroupName, ADDITIONAL_BID_PUBLIC_KEY);

  await runAdditionalBidTest(
    test, uuid, [buyer1, buyer2], auctionNonce,
    additionalBidHelper.fetchAdditionalBids(
        seller, [additionalBid1, additionalBid2]),
    /*highestScoringOtherBid=*/1.99,
    /*winningAdditionalBidId=*/'planes');
}, 'higher additional bid is filtered by negative targeting, but it has an ' +
   'invalid signature, so it still wins');

// A test of an additional bid with multiple negative interest groups.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const negativeInterestGroupName1 = 'already-owns-a-plane';
  const negativeInterestGroupName2 = 'another-negative-interest-group';

  const buyer1 = OTHER_ORIGIN1;
  const additionalBid1 = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer1, 'horses', 1.99);
  additionalBid1.auctionNonce = auctionNonce;
  additionalBidHelper.setAuctionNonceInHeader(additionalBid1, auctionNonce);

  const buyer2 = OTHER_ORIGIN2;
  const additionalBid2 = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer2, 'planes', 2.99);
  additionalBid2.auctionNonce = auctionNonce;
  additionalBidHelper.setAuctionNonceInHeader(additionalBid2, auctionNonce);
  additionalBidHelper.addNegativeInterestGroups(
      additionalBid2, [negativeInterestGroupName1, negativeInterestGroupName2],
      /*joiningOrigin=*/window.location.origin);
  additionalBidHelper.signWithSecretKeys(
      additionalBid2, [ADDITIONAL_BID_SECRET_KEY]);

  await joinNegativeInterestGroup(
      test, buyer2, negativeInterestGroupName1, ADDITIONAL_BID_PUBLIC_KEY);

  await runAdditionalBidTest(
    test, uuid, [buyer1, buyer2], auctionNonce,
    additionalBidHelper.fetchAdditionalBids(
        seller, [additionalBid1, additionalBid2]),
    /*highestScoringOtherBid=*/0,
    /*winningAdditionalBidId=*/'horses');
}, 'higher additional bid is filtered by negative targeting by two negative ' +
   'interest groups, and since one is on the device, the lower bid wins');

// Same as above, but with a mismatched joining origin.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const negativeInterestGroupName1 = 'already-owns-a-plane';
  const negativeInterestGroupName2 = 'another-negative-interest-group';

  const buyer1 = OTHER_ORIGIN1;
  const additionalBid1 = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer1, 'horses', 1.99);
  additionalBid1.auctionNonce = auctionNonce;
  additionalBidHelper.setAuctionNonceInHeader(additionalBid1, auctionNonce);

  const buyer2 = OTHER_ORIGIN2;
  const additionalBid2 = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer2, 'planes', 2.99);
  additionalBid2.auctionNonce = auctionNonce;
  additionalBidHelper.setAuctionNonceInHeader(additionalBid2, auctionNonce);
  additionalBidHelper.addNegativeInterestGroups(
      additionalBid2, [negativeInterestGroupName1, negativeInterestGroupName2],
      /*joiningOrigin=*/OTHER_ORIGIN1);
  additionalBidHelper.signWithSecretKeys(
      additionalBid2, [ADDITIONAL_BID_SECRET_KEY]);

  await joinNegativeInterestGroup(
      test, buyer2, negativeInterestGroupName1, ADDITIONAL_BID_PUBLIC_KEY);

  await runAdditionalBidTest(
    test, uuid, [buyer1, buyer2], auctionNonce,
    additionalBidHelper.fetchAdditionalBids(
        seller, [additionalBid1, additionalBid2]),
    /*highestScoringOtherBid=*/1.99,
    /*winningAdditionalBidId=*/'planes');
}, 'higher additional bid is filtered by negative targeting by two negative ' +
   'interest groups, but because of a joining origin mismatch, it still wins');

// Ensure that trusted seller signals are retrieved for additional bids.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const buyer = OTHER_ORIGIN1;
  const additionalBid = additionalBidHelper.createAdditionalBid(
      uuid, seller, buyer, 'horses', 1.99);
  additionalBid.auctionNonce = auctionNonce;
  additionalBidHelper.setAuctionNonceInHeader(additionalBid, auctionNonce);

  let renderURL = createRenderURL(uuid);
  await runBasicFledgeTestExpectingWinner(
      test, uuid,
      { interestGroupBuyers: [buyer],
        auctionNonce: auctionNonce,
        additionalBids: additionalBidHelper.fetchAdditionalBids(
            seller, [additionalBid]),
        decisionLogicURL: createDecisionScriptURL(
            uuid,
            { scoreAd:
                `if(!"${renderURL}" in trustedScoringSignals.renderURL) ` +
                  'throw "missing trusted signals";'}),
        trustedScoringSignalsURL: TRUSTED_SCORING_SIGNALS_URL});
}, 'trusted seller signals retrieved for additional bids');
