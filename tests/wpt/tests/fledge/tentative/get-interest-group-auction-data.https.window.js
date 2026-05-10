// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/ba-fledge-util.sub.js
// META: script=resources/fledge-util.sub.js
// META: script=third_party/cbor-js/cbor.js
// META: script=/common/subset-tests.js
// META: timeout=long

"use strict";

// These tests focus on the navigator.getInterestGroupAdAuctionData() method.

subsetTest(promise_test, async test => {
  const result = await navigator.getInterestGroupAdAuctionData({
    coordinatorOrigin: await BA.configureCoordinator(),
    seller: window.location.origin
  });
  assert_true(result.requestId !== null);
  assert_true(result.request.length === 0);
}, 'getInterestGroupAdAuctionData() with no interest groups returns a zero length result.');

async function testInvalidConfig(test, configObj, desc) {
  if (!configObj.coordinatorOrigin) {
    configObj.coordinatorOrigin = await BA.configureCoordinator();
  }
  await promise_rejects_js(
      test, TypeError, navigator.getInterestGroupAdAuctionData(configObj),
      desc);
}

subsetTest(promise_test, async test => {
  await testInvalidConfig(test, {}, 'no seller');
  await testInvalidConfig(test, {seller: 'example'}, 'invalid seller 1');
  await testInvalidConfig(
      test, {seller: 'http://example.org'}, 'invalid seller 2');
  await testInvalidConfig(
      test, {seller: 'https://example.org', coordinatorOrigin: 'example'},
      'invalid coordinator 1');
  await testInvalidConfig(
      test, {seller: 'https://example.org', coordinatorOrigin: 'example.org'},
      'invalid coordinator 2');

  await testInvalidConfig(
      test, {seller: 'https://example.org', perBuyerConfig: {'a': {}}},
      'invalid buyer 1');

  await testInvalidConfig(
      test,
      {seller: 'https://example.org', perBuyerConfig: {'http://a.com': {}}},
      'invalid buyer 2');

  await testInvalidConfig(
      test, {
        seller: 'https://example.org',
        perBuyerConfig: {'https://a.com': {}, 'http://b.com': {}}
      },
      'invalid buyer 3');

  await testInvalidConfig(
      test, {
        seller: 'https://example.org',
        perBuyerConfig: {'https://a.com': {}, 'https://b.com': {}}
      },
      'missing size info w/per-buyer config 1');

  await testInvalidConfig(
      test, {
        seller: 'https://example.org',
        perBuyerConfig:
            {'https://a.com': {targetSize: 400}, 'https://b.com': {}}
      },
      'missing size info w/per-buyer config 2');
}, 'getInterestGroupAdAuctionData() config checks');

// Returns an AuctionAdInterestGroup that sets all fields that can be exported
// via getInterestGroupAdAuctionData().
function makeTemplateIgConfig(uuid) {
  const adsArray = [
    {
      renderURL: createRenderURL(uuid) + '&a',
      adRenderId: 'a',
      metadata: 'ada',
      sizeGroup: 'small'
    },
    {
      renderURL: createRenderURL(uuid) + '&b',
      adRenderId: 'b',
      metadata: 'adb',
      sizeGroup: 'big'
    }
  ];
  const adComponentsArray = [
    {
      renderURL: 'https://example.org/ca',
      adRenderId: 'ca',
      metadata: 'compa',
      sizeGroup: 'big'
    },
    {
      renderURL: 'https://example.org/cb',
      adRenderId: 'cb',
      metadata: 'compb',
      sizeGroup: 'small'
    },
    {
      renderURL: 'https://example.org/cc',
      adRenderId: 'cc',
      metadata: 'compc',
      sizeGroup: 'big'
    },
  ];
  return {
    ads: adsArray,
    adComponents: adComponentsArray,
    adSizes: {
      's': {width: '100px', height: '30px'},
      'xl': {width: '1000px', height: '300px'}
    },
    sizeGroups: {'small': ['s'], 'big': ['xl']},
    trustedBiddingSignalsKeys: ['alpha', 'beta'],
    userBiddingSignals: 14
  };
}

/*************************************************************************
 * Tests for the multi-seller variant of the API
 *************************************************************************/

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const igConfig = makeTemplateIgConfig(uuid);
  await joinInterestGroup(test, uuid, igConfig);

  await promise_rejects_js(test, TypeError, navigator.getInterestGroupAdAuctionData({
    sellers: [{
        coordinatorOrigin: await BA.configureCoordinator(),
        seller: window.location.origin,
      }, {
        coordinatorOrigin: await BA.configureCoordinator(),
        seller: "http://not.secure.test/",
    }]
  }));
}, 'getInterestGroupAdAuctionData() multi-seller with multiple sellers - one invalid seller');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const igConfig = makeTemplateIgConfig(uuid);
  await joinInterestGroup(test, uuid, igConfig);

  await promise_rejects_js(test, TypeError, navigator.getInterestGroupAdAuctionData({
  }));
}, 'getInterestGroupAdAuctionData() one of "seller" and "sellers" is required');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const igConfig = makeTemplateIgConfig(uuid);
  await joinInterestGroup(test, uuid, igConfig);

  await promise_rejects_js(test, TypeError, navigator.getInterestGroupAdAuctionData({
    seller: window.location.origin,
    sellers: [{
      coordinatorOrigin: await BA.configureCoordinator(),
      seller: window.location.origin,
    }]
  }));
}, 'getInterestGroupAdAuctionData() doesn\'t allow "seller" and "sellers" fields');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const igConfig = makeTemplateIgConfig(uuid);
  await joinInterestGroup(test, uuid, igConfig);

  await promise_rejects_js(test, TypeError, navigator.getInterestGroupAdAuctionData({
    coordinatorOrigin: window.location.origin,
    sellers: [{
      coordinatorOrigin: await BA.configureCoordinator(),
      seller: window.location.origin,
    }]
  }));
}, 'getInterestGroupAdAuctionData() doesn\'t allow "coordinatorOrigin" and "sellers" fields');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const igConfig = makeTemplateIgConfig(uuid);
  await joinInterestGroup(test, uuid, igConfig);

  await promise_rejects_js(test, TypeError, navigator.getInterestGroupAdAuctionData({
    sellers: [{
      coordinatorOrigin: await BA.configureCoordinator(),
      seller: window.location.origin,
    }, {
      coordinatorOrigin: await BA.configureCoordinator(),
      seller: window.location.origin,
    }
  ]
  }));
}, 'getInterestGroupAdAuctionData() doesn\'t allow duplicate sellers in "sellers" field');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const igConfig = makeTemplateIgConfig(uuid);
  await joinInterestGroup(test, uuid, igConfig);

  const result = await navigator.getInterestGroupAdAuctionData({
    sellers: []
  });
  assert_equals(result.requestId, "");
  assert_array_equals(result.requests, []);
}, 'getInterestGroupAdAuctionData() with no sellers');
