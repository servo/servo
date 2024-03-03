// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-last

"use strict;"

// This file contains tests for additional bids and negative targeting.
//
// TODO:
// - test that an negatively targeted additional bid is suppressed.
// - test that an incorrectly signed additional bid is not negative targeted.
// - test that an missing-signature additional bid is not negative targeted.
// - test that an additional bid with some correct signatures can be negative.
//       negative targeted for those negative interest groups whose signatures
//       match.
// - test an additional bid with multiple negative interest groups.
// - test that multiple negative interest groups with mismatched joining origins
//       is not negative targeted.
// - test that additional bids can be fetched using an iframe navigation.
// - test that additional bids are not fetched using an iframe navigation for
//      which the `adAuctionHeaders=true` attribute is not specified.
// - test that additional bids are not fetched using a Fetch request for which
//      `adAuctionHeaders: true` is not specified.
// - test that an additional bid with an incorrect auction nonce is not used
//       included in an auction. Same for seller and top-level seller.
// - test that correctly formatted additional bids are included in an auction
//       when fetched alongside malformed additional bid headers by a Fetch
//       request.
// - test that correctly formatted additional bids are included in an auction
//       when fetched alongside malformed additional bid headers by an iframe
//       navigation.
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
// - test that a malformed additional bid causes that one additional bid to be
//       ignored, but the rest of the auction (and other additional bids, even
//       from the same fetch) continue on.
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

// Single-seller auction with a single buyer who places a single additional
// bid. As the only bid, this wins.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const buyer = OTHER_ORIGIN1;
  const additionalBid = createAdditionalBid(
      uuid, auctionNonce, seller, buyer, 'horses', 1.99);

  await runAdditionalBidTest(
      test, uuid, [buyer], auctionNonce,
      fetchAdditionalBids(seller, [additionalBid]),
      /*highestScoringOtherBid=*/0,
      /*winningAdditionalBidId=*/'horses');
}, 'single valid additional bid');

// Single-seller auction with a two buyers competing with additional bids.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const auctionNonce = await navigator.createAuctionNonce();
  const seller = SINGLE_SELLER_AUCTION_SELLER;

  const buyer1 = OTHER_ORIGIN1;
  const additionalBid1 = createAdditionalBid(
      uuid, auctionNonce, seller, buyer1, 'horses', 1.99);

  const buyer2 = OTHER_ORIGIN2;
  const additionalBid2 = createAdditionalBid(
      uuid, auctionNonce, seller, buyer2, 'planes', 2.99);

  await runAdditionalBidTest(
      test, uuid, [buyer1, buyer2], auctionNonce,
      fetchAdditionalBids(seller, [additionalBid1, additionalBid2]),
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
  const additionalBid1 = createAdditionalBid(
      uuid, auctionNonce, seller, buyer1, 'horses', 1.99);

  const buyer2 = OTHER_ORIGIN2;
  const additionalBid2 = createAdditionalBid(
      uuid, auctionNonce, seller, buyer2, 'planes', 2.99);


  await runAdditionalBidTest(
    test, uuid, [buyer1, buyer2], auctionNonce,
    Promise.all([
        fetchAdditionalBids(seller, [additionalBid1]),
        fetchAdditionalBids(seller, [additionalBid2])
    ]),
    /*highestScoringOtherBid=*/1.99,
    /*winningAdditionalBidId=*/'planes');
}, 'two valid additional bids from two distinct Fetch requests');
