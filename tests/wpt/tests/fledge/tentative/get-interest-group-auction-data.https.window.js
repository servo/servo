// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-4

// These tests focus on the navigator.getInterestGroupAdAuctionData() method.

subsetTest(promise_test, async test => {
  const result = await navigator.getInterestGroupAdAuctionData({ seller: window.location.origin });
  assert_true(result.requestId !== null);
  assert_true(result.request.length === 0);
}, 'getInterestGroupAdAuctionData() with no interest groups returns a zero length result.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid);

  const result = await navigator.getInterestGroupAdAuctionData({ seller: window.location.origin });
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);
}, 'getInterestGroupAdAuctionData() with one interest group returns a non-zero length result.');
