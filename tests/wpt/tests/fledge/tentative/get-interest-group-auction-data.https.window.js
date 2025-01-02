// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/ba-fledge-util.sub.js
// META: script=resources/fledge-util.sub.js
// META: script=third_party/cbor-js/cbor.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-4
// META: variant=?5-8
// META: variant=?9-12
// META: variant=?13-16

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

  // These two actually succeed.
  let result = await navigator.getInterestGroupAdAuctionData({
    coordinatorOrigin: await BA.configureCoordinator(),
    seller: 'https://example.org',
    perBuyerConfig:
        {'https://a.com': {targetSize: 400}, 'https://b.com': {targetSize: 500}}
  });
  assert_true(result.requestId !== null);

  result = await navigator.getInterestGroupAdAuctionData({
    coordinatorOrigin: await BA.configureCoordinator(),
    seller: 'https://example.org',
    perBuyerConfig: {'https://a.com': {targetSize: 400}, 'https://b.com': {}},
    requestSize: 5000
  });
  assert_true(result.requestId !== null);
}, 'getInterestGroupAdAuctionData() config checks');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid);

  const result = await navigator.getInterestGroupAdAuctionData({
    coordinatorOrigin: await BA.configureCoordinator(),
    seller: window.location.origin
  });
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
  assert_equals(ig.browserSignals.bidCount, 0, 'bidCount');
  assert_array_equals(ig.browserSignals.prevWins, []);
}, 'getInterestGroupAdAuctionData() with one interest group returns a valid result.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid, {
    ads: [
      {renderURL: createRenderURL(uuid) + '&a', adRenderId: 'a'},
      {renderURL: createRenderURL(uuid) + '&a', adRenderId: 'b'}
    ]
  });

  const result = await navigator.getInterestGroupAdAuctionData({
    coordinatorOrigin: await BA.configureCoordinator(),
    seller: window.location.origin
  });
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);
  let ig = validateWithOneIg(decoded);

  // This ig should have two ads with adRenderIds, but not URLs.
  assert_equals(ig.name, DEFAULT_INTEREST_GROUP_NAME);
  assert_array_equals(ig.ads, ['a', 'b']);
  assert_equals(ig.browserSignals.joinCount, 1, 'joinCount');
  assert_equals(ig.browserSignals.bidCount, 0, 'bidCount');
  assert_array_equals(ig.browserSignals.prevWins, []);
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

  const result = await navigator.getInterestGroupAdAuctionData({
    coordinatorOrigin: await BA.configureCoordinator(),
    seller: window.location.origin
  });
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

  const result = await navigator.getInterestGroupAdAuctionData({
    coordinatorOrigin: await BA.configureCoordinator(),
    seller: window.location.origin
  });
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

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const igConfig = makeTemplateIgConfig(uuid);
  igConfig.auctionServerRequestFlags = ['include-full-ads'];
  await joinInterestGroup(test, uuid, igConfig);

  const result = await navigator.getInterestGroupAdAuctionData({
    coordinatorOrigin: await BA.configureCoordinator(),
    seller: window.location.origin
  });
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);
  let ig = validateWithOneIg(decoded);

  assert_equals(ig.name, DEFAULT_INTEREST_GROUP_NAME);
  assert_true(ig.ads instanceof Array);
  assert_equals(ig.ads.length, 2, '# of ads');
  assert_equals(ig.ads[0].renderURL, igConfig.ads[0].renderURL, 'renderURL 0');
  assert_equals(ig.ads[1].renderURL, igConfig.ads[1].renderURL, 'renderURL 1');
  assert_equals(ig.ads[0].adRenderId, 'a', 'adRenderId 0');
  assert_equals(ig.ads[1].adRenderId, 'b', 'adRenderId 1');
  assert_equals(ig.ads[0].metadata, '"ada"', 'metadata 0');
  assert_equals(ig.ads[1].metadata, '"adb"', 'metadata 1');
  assert_equals(ig.ads[0].sizeGroup, 'small', 'sizegroup 0');
  assert_equals(ig.ads[1].sizeGroup, 'big', 'sizegroup 1');

  assert_true(ig.components instanceof Array);
  assert_equals(ig.components.length, 3, '# of component ads');
  assert_equals(
      ig.components[0].renderURL, igConfig.adComponents[0].renderURL,
      'component renderURL 0');
  assert_equals(
      ig.components[1].renderURL, igConfig.adComponents[1].renderURL,
      'component renderURL 1');
  assert_equals(
      ig.components[2].renderURL, igConfig.adComponents[2].renderURL,
      'component renderURL 2');
  assert_equals(ig.components[0].adRenderId, 'ca', 'component adRenderId 0');
  assert_equals(ig.components[1].adRenderId, 'cb', 'component adRenderId 1');
  assert_equals(ig.components[2].adRenderId, 'cc', 'component adRenderId 2');
  assert_equals(ig.components[0].metadata, '"compa"', 'component metadata 0');
  assert_equals(ig.components[1].metadata, '"compb"', 'component metadata 1');
  assert_equals(ig.components[2].metadata, '"compc"', 'component metadata 2');
  assert_equals(ig.components[0].sizeGroup, 'big', 'component sizegroup 0');
  assert_equals(ig.components[1].sizeGroup, 'small', 'component sizegroup 1');
  assert_equals(ig.components[2].sizeGroup, 'big', 'component sizegroup 2');

  assert_true(ig.biddingSignalsKeys instanceof Array);
  assert_array_equals(ig.biddingSignalsKeys, ['alpha', 'beta']);
  assert_equals(ig.userBiddingSignals, '14');
}, 'getInterestGroupAdAuctionData() all IG data fields, with include-full-ads');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const igConfig = makeTemplateIgConfig(uuid);
  await joinInterestGroup(test, uuid, igConfig);

  const result = await navigator.getInterestGroupAdAuctionData({
    coordinatorOrigin: await BA.configureCoordinator(),
    seller: window.location.origin
  });
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);
  let ig = validateWithOneIg(decoded);

  assert_equals(ig.name, DEFAULT_INTEREST_GROUP_NAME);
  assert_array_equals(ig.ads, ['a', 'b']);

  assert_true(ig.components instanceof Array);
  assert_array_equals(ig.ads, ['a', 'b']);
  assert_array_equals(ig.components, ['ca', 'cb', 'cc']);

  assert_array_equals(ig.biddingSignalsKeys, ['alpha', 'beta']);
  assert_equals(ig.userBiddingSignals, '14');
}, 'getInterestGroupAdAuctionData() all IG data fields, w/o include-full-ads');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  const igConfig = makeTemplateIgConfig(uuid);
  igConfig.auctionServerRequestFlags = ['omit-user-bidding-signals'];
  await joinInterestGroup(test, uuid, igConfig);

  const result = await navigator.getInterestGroupAdAuctionData({
    coordinatorOrigin: await BA.configureCoordinator(),
    seller: window.location.origin
  });
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);
  let ig = validateWithOneIg(decoded);

  assert_equals(ig.name, DEFAULT_INTEREST_GROUP_NAME);
  assert_array_equals(ig.ads, ['a', 'b']);

  assert_true(ig.components instanceof Array);
  assert_array_equals(ig.ads, ['a', 'b']);
  assert_array_equals(ig.components, ['ca', 'cb', 'cc']);

  assert_array_equals(ig.biddingSignalsKeys, ['alpha', 'beta']);
  assert_false('userBiddingSignals' in ig, 'userBiddingSignals');
}, 'getInterestGroupAdAuctionData() all IG data fields, with omit-user-bidding-signals');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  const igConfig = makeTemplateIgConfig(uuid);

  // Join twice.
  await joinInterestGroup(test, uuid, igConfig);
  await joinInterestGroup(test, uuid, igConfig);

  // And run an auction. This is a local auction, not a B&A one, run to update
  // bid/win stats.
  await runBasicFledgeAuctionAndNavigate(test, uuid);
  await waitForObservedRequests(
      uuid, [createBidderReportURL(uuid), createSellerReportURL(uuid)]);

  const result = await navigator.getInterestGroupAdAuctionData({
    coordinatorOrigin: await BA.configureCoordinator(),
    seller: window.location.origin
  });
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);
  let ig = validateWithOneIg(decoded);
  assert_equals(ig.browserSignals.joinCount, 2, 'joinCount');
  assert_equals(ig.browserSignals.bidCount, 1, 'bidCount');

  // RecencyMs is the # of milliseconds since the join. We can't exactly say
  // what it is, but it shouldn't be too huge.
  assert_true(typeof ig.browserSignals.recencyMs === 'number');
  assert_between_inclusive(
      ig.browserSignals.recencyMs, 0, 60000,
      'RecencyMs is between 0 and 60 seconds');
  // It's also supposed to be an integer.
  assert_equals(
      ig.browserSignals.recencyMs, Math.round(ig.browserSignals.recencyMs),
      'RecencyMs is an integer');

  // One win. The format here depends highly on whether full ads are used or
  // not.
  assert_true(
      ig.browserSignals.prevWins instanceof Array, 'prevWins is an array');
  assert_equals(ig.browserSignals.prevWins.length, 1, 'prevWins length');
  assert_true(
      ig.browserSignals.prevWins[0] instanceof Array,
      'prevWins[0] is an array');
  assert_equals(ig.browserSignals.prevWins[0].length, 2, 'prevWins[0] length');

  // prevWins[0][0] is the time delta in second again.
  let prevWinTime = ig.browserSignals.prevWins[0][0];
  assert_true(typeof prevWinTime === 'number');
  assert_between_inclusive(
      prevWinTime, 0, 60, 'prevWinTime is between 0 and 60 seconds');
  // It's also supposed to be an integer.
  assert_equals(
      prevWinTime, Math.round(prevWinTime), 'prevWinTime is an integer');

  // prevWins[0][1] is the adRenderId of the winner.
  assert_equals(ig.browserSignals.prevWins[0][1], 'a');
}, 'getInterestGroupAdAuctionData() browserSignals');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  const igConfig = makeTemplateIgConfig(uuid);
  igConfig.auctionServerRequestFlags = ['include-full-ads'];

  // Join twice.
  await joinInterestGroup(test, uuid, igConfig);
  await joinInterestGroup(test, uuid, igConfig);

  // And run an auction. This is a local auction, not a B&A one, run to update
  // bid/win stats.
  await runBasicFledgeAuctionAndNavigate(test, uuid);
  await waitForObservedRequests(
      uuid, [createBidderReportURL(uuid), createSellerReportURL(uuid)]);

  const result = await navigator.getInterestGroupAdAuctionData({
    coordinatorOrigin: await BA.configureCoordinator(),
    seller: window.location.origin
  });
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);
  let ig = validateWithOneIg(decoded);
  assert_equals(ig.browserSignals.joinCount, 2, 'joinCount');
  assert_equals(ig.browserSignals.bidCount, 1, 'bidCount');

  // RecencyMs is the # of milliseconds since the join. We can't exactly say
  // what it is, but it shouldn't be too huge.
  assert_true(typeof ig.browserSignals.recencyMs === 'number');
  assert_between_inclusive(
      ig.browserSignals.recencyMs, 0, 60000,
      'RecencyMs is between 0 and 60 seconds');
  // It's also supposed to be an integer.
  assert_equals(
      ig.browserSignals.recencyMs, Math.round(ig.browserSignals.recencyMs),
      'RecencyMs is an integer');

  // One win. The format here depends highly on whether full ads are used or
  // not.
  assert_true(
      ig.browserSignals.prevWins instanceof Array, 'prevWins is an array');
  assert_equals(ig.browserSignals.prevWins.length, 1, 'prevWins length');
  assert_true(
      ig.browserSignals.prevWins[0] instanceof Array,
      'prevWins[0] is an array');
  assert_equals(ig.browserSignals.prevWins[0].length, 2, 'prevWins[0] length');

  // prevWins[0][0] is the time delta in second again.
  let prevWinTime = ig.browserSignals.prevWins[0][0];
  assert_true(typeof prevWinTime === 'number');
  assert_between_inclusive(
      prevWinTime, 0, 60, 'prevWinTime is between 0 and 60 seconds');
  // It's also supposed to be an integer.
  assert_equals(
      prevWinTime, Math.round(prevWinTime), 'prevWinTime is an integer');

  // prevWins[0][1] is an ad object w/include-full-ads on (with renderURL,
  // metadata, and adRenderId).
  let prevWinAd = ig.browserSignals.prevWins[0][1];
  assert_equals(
      prevWinAd.renderURL, igConfig.ads[0].renderURL, 'prevWin ad renderURL');
  assert_equals(prevWinAd.metadata, '"ada"', 'prevWin ad metadata');
  assert_equals(prevWinAd.adRenderId, 'a', 'prevWin ad adRenderId');
}, 'getInterestGroupAdAuctionData() browserSignals with include-full-ads');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  const igTemplate = makeTemplateIgConfig(uuid);
  await joinInterestGroup(test, uuid, {...igTemplate, name: 'first'});
  await joinInterestGroup(test, uuid, {...igTemplate, name: 'second'});

  const result = await navigator.getInterestGroupAdAuctionData({
    coordinatorOrigin: await BA.configureCoordinator(),
    seller: window.location.origin
  });
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);
  assert_equals(decoded.message.version, 0);
  assert_equals(decoded.message.publisher, window.location.hostname);
  assert_equals(typeof decoded.message.generationId, 'string');

  let origin = window.location.origin;
  let igMapKeys = Object.getOwnPropertyNames(decoded.message.interestGroups);
  assert_array_equals(igMapKeys, [origin]);
  assert_equals(decoded.message.interestGroups[origin].length, 2);
  let names = [
    decoded.message.interestGroups[origin][0].name,
    decoded.message.interestGroups[origin][1].name
  ];
  assert_array_equals(names.sort(), ['first', 'second']);
}, 'getInterestGroupAdAuctionData() with multiple interest groups');

async function joinCrossOriginIG(test, uuid, origin, name) {
  let iframe = await createIframe(test, origin, 'join-ad-interest-group');
  await runInFrame(
      test, iframe,
      `await joinInterestGroup(test_instance, "${uuid}", {name: "${name}"});`);
}

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinCrossOriginIG(test, uuid, OTHER_ORIGIN1, 'o1');
  await joinCrossOriginIG(test, uuid, OTHER_ORIGIN2, 'o2');
  await joinCrossOriginIG(test, uuid, OTHER_ORIGIN3, 'o3');
  await joinCrossOriginIG(test, uuid, OTHER_ORIGIN4, 'o4');

  const result = await navigator.getInterestGroupAdAuctionData({
    coordinatorOrigin: await BA.configureCoordinator(),
    seller: window.location.origin
  });
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);
  assert_equals(decoded.message.version, 0);
  assert_equals(decoded.message.publisher, window.location.hostname);
  assert_equals(typeof decoded.message.generationId, 'string');
  let igMapKeys = Object.getOwnPropertyNames(decoded.message.interestGroups);
  assert_array_equals(
      igMapKeys.sort(),
      [OTHER_ORIGIN1, OTHER_ORIGIN2, OTHER_ORIGIN3, OTHER_ORIGIN4].sort());
  assert_equals(decoded.message.interestGroups[OTHER_ORIGIN1].length, 1);
  assert_equals(decoded.message.interestGroups[OTHER_ORIGIN1][0].name, 'o1');
  assert_equals(decoded.message.interestGroups[OTHER_ORIGIN2].length, 1);
  assert_equals(decoded.message.interestGroups[OTHER_ORIGIN2][0].name, 'o2');
  assert_equals(decoded.message.interestGroups[OTHER_ORIGIN3].length, 1);
  assert_equals(decoded.message.interestGroups[OTHER_ORIGIN3][0].name, 'o3');
  assert_equals(decoded.message.interestGroups[OTHER_ORIGIN4].length, 1);
  assert_equals(decoded.message.interestGroups[OTHER_ORIGIN4][0].name, 'o4');
}, 'getInterestGroupAdAuctionData() with multiple buyers');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinCrossOriginIG(test, uuid, OTHER_ORIGIN1, 'o1');
  await joinCrossOriginIG(test, uuid, OTHER_ORIGIN2, 'o2');
  await joinCrossOriginIG(test, uuid, OTHER_ORIGIN3, 'o3');
  await joinCrossOriginIG(test, uuid, OTHER_ORIGIN4, 'o4');

  let config = {
    coordinatorOrigin: await BA.configureCoordinator(),
    seller: window.location.origin,
    perBuyerConfig: {},
    requestSize: 5000
  };
  config.perBuyerConfig[OTHER_ORIGIN2] = {};
  config.perBuyerConfig[OTHER_ORIGIN3] = {};
  const result = await navigator.getInterestGroupAdAuctionData(config);
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);
  assert_equals(decoded.message.version, 0);
  assert_equals(decoded.message.publisher, window.location.hostname);
  assert_equals(typeof decoded.message.generationId, 'string');
  let igMapKeys = Object.getOwnPropertyNames(decoded.message.interestGroups);
  assert_array_equals(igMapKeys.sort(), [OTHER_ORIGIN2, OTHER_ORIGIN3].sort());
  assert_equals(decoded.message.interestGroups[OTHER_ORIGIN2].length, 1);
  assert_equals(decoded.message.interestGroups[OTHER_ORIGIN2][0].name, 'o2');
  assert_equals(decoded.message.interestGroups[OTHER_ORIGIN3].length, 1);
  assert_equals(decoded.message.interestGroups[OTHER_ORIGIN3][0].name, 'o3');
}, 'getInterestGroupAdAuctionData() uses perBuyerConfig to select buyers');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid);

  const result = await navigator.getInterestGroupAdAuctionData({
    coordinatorOrigin: await BA.configureCoordinator(),
    seller: window.location.origin
  });
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);

  assert_own_property(decoded.message, 'enforceKAnon');
  assert_equals(decoded.message.enforceKAnon, true);
}, 'getInterestGroupAdAuctionData() requests k-anon.');
