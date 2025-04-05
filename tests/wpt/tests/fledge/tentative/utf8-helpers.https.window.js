// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-5
// META: variant=?6-10
// META: variant=?11-15

'use strict';

// These tests cover encodeUtf8 and decodeUtf8.

const helpers = `
    function assertEq(l, r, label) {
      if (l !== r)
        throw 'Mismatch ' + label;
    }

    function assertByteArray(result, expect) {
      if (!(result instanceof Uint8Array)) {
        throw 'Not a Uint8Array!';
      }
      assertEq(result.length, expect.length, 'length');
      for (var i = 0; i < result.length; ++i) {
        assertEq(result[i], expect[i], i);
      }
    }

    function assertString(result, expect) {
      if (typeof result !== 'string') {
        throw 'Not a string';
      }
      assertEq(result.length, expect.length, 'length');
      for (var i = 0; i < result.length; ++i) {
        assertEq(result.charCodeAt(i), expect.charCodeAt(i), i);
      }
    }
`

async function testConversion(test, conversionBody) {
  const uuid = generateUuid(test);
  let sellerReportURL = createSellerReportURL(uuid);
  let bidderReportURL = createBidderReportURL(uuid);

  let fullBody = `
    ${helpers}
    ${conversionBody}
  `;

  let biddingLogicURL = createBiddingScriptURL({
    generateBid: fullBody,
    reportWin: fullBody + `sendReportTo('${bidderReportURL}')`
  });

  let decisionLogicURL = createDecisionScriptURL(uuid, {
    scoreAd: fullBody,
    reportResult: fullBody + `sendReportTo('${sellerReportURL}')`
  });

  await joinInterestGroup(test, uuid, {biddingLogicURL: biddingLogicURL});
  await runBasicFledgeAuctionAndNavigate(
      test, uuid, {decisionLogicURL: decisionLogicURL});
  await waitForObservedRequests(uuid, [sellerReportURL, bidderReportURL]);
}

async function testConversionException(test, conversionBody) {
  const uuid = generateUuid(test);
  let sellerReportURL = createSellerReportURL(uuid);
  let bidderReportURL = createBidderReportURL(uuid);

  let fullBody = `
    ${helpers}
    try {
      ${conversionBody};
      return -1;
    } catch (e) {
    }
  `;

  let biddingLogicURL = createBiddingScriptURL({
    generateBid: fullBody,
    reportWin: fullBody + `sendReportTo('${bidderReportURL}')`
  });

  let decisionLogicURL = createDecisionScriptURL(uuid, {
    scoreAd: fullBody,
    reportResult: fullBody + `sendReportTo('${sellerReportURL}')`
  });

  await joinInterestGroup(test, uuid, {biddingLogicURL: biddingLogicURL});
  await runBasicFledgeAuctionAndNavigate(
      test, uuid, {decisionLogicURL: decisionLogicURL});
  await waitForObservedRequests(uuid, [sellerReportURL, bidderReportURL]);
}

subsetTest(promise_test, async test => {
  await testConversion(
      test, `let result = protectedAudience.encodeUtf8('ABC\u0490');
             assertByteArray(result, [65, 66, 67, 0xD2, 0x90])`);
}, 'encodeUtf8 - basic');

subsetTest(promise_test, async test => {
  await testConversion(
      test, `let result = protectedAudience.encodeUtf8('A\uD800C');
             assertByteArray(result, [65, 0xEF, 0xBF, 0xBD, 67])`);
}, 'encodeUtf8 - mismatched surrogate gets replaced');

subsetTest(promise_test, async test => {
  await testConversion(
      test, `let result = protectedAudience.encodeUtf8('A\uD83D\uDE02C');
             assertByteArray(result, [65, 0xF0, 0x9F, 0x98, 0x82, 67])`);
}, 'encodeUtf8 - surrogate pair combined');

subsetTest(promise_test, async test => {
  const conversionBody = `
    let obj = {
      toString: () => "ABC"
    };
    let result = protectedAudience.encodeUtf8(obj);
    assertByteArray(result, [65, 66, 67])
  `;
  await testConversion(test, conversionBody);
}, 'encodeUtf8 - custom string conversion');

subsetTest(promise_test, async test => {
  const conversionBody = `
    let result = protectedAudience.encodeUtf8();
  `;
  await testConversionException(test, conversionBody);
}, 'encodeUtf8 - not enough arguments');

subsetTest(promise_test, async test => {
  const conversionBody = `
    let obj = {
      toString: () => { throw 'no go' }
    };
    let result = protectedAudience.encodeUtf8(obj);
  `;
  await testConversionException(test, conversionBody);
}, 'encodeUtf8 - custom string conversion failure');

subsetTest(promise_test, async test => {
  const conversionBody = `
    let input = new Uint8Array([65, 66, 0xD2, 0x90, 67]);
    let result = protectedAudience.decodeUtf8(input);
    assertString(result, 'AB\u0490C');
  `;
  await testConversion(test, conversionBody);
}, 'decodeUtf8 - basic');

subsetTest(promise_test, async test => {
  const conversionBody = `
    let input = new Uint8Array([65, 32, 0xD2]);
    let result = protectedAudience.decodeUtf8(input);
    if (result.indexOf('\uFFFD') === -1)
      throw 'Should have replacement character';
  `;
  await testConversion(test, conversionBody);
}, 'decodeUtf8 - broken utf-8');

subsetTest(promise_test, async test => {
  const conversionBody = `
    let input = new Uint8Array([65, 32, 0xED, 0xA0, 0x80, 66]);
    let result = protectedAudience.decodeUtf8(input);
    if (result.indexOf('\uFFFD') === -1)
      throw 'Should have replacement character';
  `;
  await testConversion(test, conversionBody);
}, 'decodeUtf8 - mismatched surrogate');

subsetTest(promise_test, async test => {
  const conversionBody = `
    let input = new Uint8Array([65, 0xF0, 0x9F, 0x98, 0x82, 67]);
    let result = protectedAudience.decodeUtf8(input);
    assertString(result, 'A\uD83D\uDE02C');
  `;
  await testConversion(test, conversionBody);
}, 'decodeUtf8 - non-BMP character');

subsetTest(promise_test, async test => {
  const conversionBody = `
    let buffer = new ArrayBuffer(8);
    let fullView = new Uint8Array(buffer);
    for (let i = 0; i < fullView.length; ++i)
      fullView[i] = 65 + i;
    let partialView = new Uint8Array(buffer, 2, 3);
    assertString(protectedAudience.decodeUtf8(fullView),
                 'ABCDEFGH');
    assertString(protectedAudience.decodeUtf8(partialView),
                 'CDE');
  `;
  await testConversion(test, conversionBody);
}, 'decodeUtf8 - proper Uint8Array handling');

subsetTest(promise_test, async test => {
  const conversionBody = `
    let result = protectedAudience.decodeUtf8();
  `;
  await testConversionException(test, conversionBody);
}, 'decodeUtf8 - not enough arguments');

subsetTest(promise_test, async test => {
  const conversionBody = `
    let result = protectedAudience.decodeUtf8([65, 32, 66]);
  `;
  await testConversionException(test, conversionBody);
}, 'decodeUtf8 - wrong type');
