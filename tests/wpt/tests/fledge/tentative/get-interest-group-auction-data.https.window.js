// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/ba-fledge-util.sub.js
// META: script=resources/fledge-util.sub.js
// META: script=third_party/cbor-js/cbor.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-4
// META: variant=?5-last

// These tests focus on the navigator.getInterestGroupAdAuctionData() method.

// Checks various fields for decoded InterestGroupAdAuctionData that's
// one IG owned by this origin, and returns that IG's info.
function validateWithOneIg(decoded) {
  assert_equals(decoded.message.version, 0);
  assert_equals(decoded.message.publisher, window.location.hostname);
  assert_equals(typeof decoded.message.generationId, 'string');
  let igMapKeys = Object.getOwnPropertyNames(decoded.message.interestGroups);
  assert_array_equals(igMapKeys, [window.location.origin]);
  let igInfo = decoded.message.interestGroups[window.location.origin];
  assert_true(igInfo instanceof Array);
  assert_equals(igInfo.length, 1, 'number of IGs');
  return igInfo[0];
}

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

  let decoded = await BA.decodeInterestGroupData(result.request);

  // Check that the required field and one IG (but no ad data) are here,
  // since we neither provided adRenderIds or asked for full data to be
  // included.
  let ig = validateWithOneIg(decoded);
  assert_equals(ig.name, DEFAULT_INTEREST_GROUP_NAME);
  assert_array_equals(ig.ads, []);
  assert_equals(ig.browserSignals.joinCount, 1, 'joinCount');
}, 'getInterestGroupAdAuctionData() with one interest group returns a valid result.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid, {
    ads: [
      {renderURL: createRenderURL(uuid) + '&a', adRenderId: 'a'},
      {renderURL: createRenderURL(uuid) + '&a', adRenderId: 'b'}
    ]
  });

  const result = await navigator.getInterestGroupAdAuctionData(
      {seller: window.location.origin});
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);
  let ig = validateWithOneIg(decoded);

  // This ig should have two ads with adRenderIds, but not URLs.
  assert_equals(ig.name, DEFAULT_INTEREST_GROUP_NAME);
  assert_array_equals(ig.ads, ['a', 'b']);
  assert_equals(ig.browserSignals.joinCount, 1, 'joinCount');
}, 'getInterestGroupAdAuctionData() with one interest group with two ads w/renderIds.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid, {
    auctionServerRequestFlags: ['omit-ads'],
    ads: [
      {renderURL: createRenderURL(uuid) + '&a', adRenderId: 'a'},
      {renderURL: createRenderURL(uuid) + '&a', adRenderId: 'b'}
    ]
  });

  const result = await navigator.getInterestGroupAdAuctionData(
      {seller: window.location.origin});
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);
  let ig = validateWithOneIg(decoded);

  assert_equals(ig.name, DEFAULT_INTEREST_GROUP_NAME);
  assert_not_own_property(ig, 'ads', 'no ads expected');
  assert_equals(ig.browserSignals.joinCount, 1, 'joinCount');
}, 'getInterestGroupAdAuctionData() with one interest group with two ads w/renderIds and omit-ads.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  const adsArray = [
    {renderURL: createRenderURL(uuid) + '&a', adRenderId: 'a'},
    {renderURL: createRenderURL(uuid) + '&a', adRenderId: 'b'}
  ];
  await joinInterestGroup(
      test, uuid,
      {auctionServerRequestFlags: ['include-full-ads'], ads: adsArray});

  const result = await navigator.getInterestGroupAdAuctionData(
      {seller: window.location.origin});
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);
  let ig = validateWithOneIg(decoded);

  // Since include-full-ads is on, this gets entire objects, not just
  // adRenderId.
  assert_equals(ig.name, DEFAULT_INTEREST_GROUP_NAME);
  assert_equals(ig.ads.length, 2, '# of ads');
  assert_equals(ig.ads[0].renderURL, adsArray[0].renderURL, 'renderURL 0');
  assert_equals(ig.ads[1].renderURL, adsArray[1].renderURL, 'renderURL 1');
  assert_equals(ig.ads[0].adRenderId, adsArray[0].adRenderId, 'adRenderId 0');
  assert_equals(ig.ads[1].adRenderId, adsArray[1].adRenderId, 'adRenderId 1');
  assert_equals(ig.browserSignals.joinCount, 1, 'joinCount');
}, 'getInterestGroupAdAuctionData() with one interest group with two ads w/renderIds and include-full-ads.');
