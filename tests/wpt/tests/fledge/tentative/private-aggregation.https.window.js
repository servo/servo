// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: script=third_party/cbor-js/cbor.js
// META: timeout=long
// META: variant=?1-5
// META: variant=?6-10
// META: variant=?11-15
// META: variant=?16-20

'use strict;'

// To better isolate from private aggregation tests run in parallel,
// don't use the usual origin here.
const MAIN_ORIGIN = OTHER_ORIGIN1;
const ALT_ORIGIN = OTHER_ORIGIN4;

const MAIN_PATH = '/.well-known/private-aggregation/report-protected-audience';
const DEBUG_PATH =
    '/.well-known/private-aggregation/debug/report-protected-audience';

const ADDITIONAL_BID_PUBLIC_KEY =
    '11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo=';

const enableDebugMode = 'privateAggregation.enableDebugMode();';

// The next 3 methods are for interfacing with the test handler for
// Private Aggregation reports; adopted wholesale from Chrome-specific
// wpt_internal/private-aggregation/resources/utils.js
const resetReports = url => {
  url = `${url}?clear_stash=true`;
  const options = {
    method: 'POST',
    mode: 'no-cors',
  };
  return fetch(url, options);
};

const delay = ms => new Promise(resolve => step_timeout(resolve, ms));

async function pollReports(path, wait_for = 1, timeout = 5000 /*ms*/) {
  const targetUrl = new URL(path, MAIN_ORIGIN);
  const endTime = performance.now() + timeout;
  const outReports = [];

  do {
    const response = await fetch(targetUrl);
    assert_true(response.ok, 'pollReports() fetch response should be OK.');
    const reports = await response.json();
    outReports.push(...reports);
    if (outReports.length >= wait_for) {
      break;
    }
    await delay(/*ms=*/ 100);
  } while (performance.now() < endTime);

  return outReports.length ? outReports : null;
};

function decodeBase64(inStr) {
  let strBytes = atob(inStr);
  let arrBytes = new Uint8Array(strBytes.length);
  for (let i = 0; i < strBytes.length; ++i) {
    arrBytes[i] = strBytes.codePointAt(i);
  }
  return arrBytes.buffer;
}

function byteArrayToBigInt(inArray) {
  let out = 0n;
  for (let byte of inArray) {
    out = out * 256n + BigInt(byte);
  }
  return out;
}

async function getDebugSamples(path) {
  const debugReports = await pollReports(path);

  let samplesDict = new Map();

  // Extract samples for debug reports, and aggregate them, so we are not
  // reliant on how aggregation happens.
  for (let jsonReport of debugReports) {
    let report = JSON.parse(jsonReport);
    for (let payload of report.aggregation_service_payloads) {
      let decoded = CBOR.decode(decodeBase64(payload.debug_cleartext_payload));
      assert_equals(decoded.operation, 'histogram');
      for (let sample of decoded.data) {
        let convertedSample = {
          bucket: byteArrayToBigInt(sample.bucket),
          value: byteArrayToBigInt(sample.value)
        };
        if (convertedSample.value !== 0n) {
          let oldCount = 0n;
          if (samplesDict.has(convertedSample.bucket)) {
            oldCount = samplesDict.get(convertedSample.bucket);
          }

          samplesDict.set(
              convertedSample.bucket, oldCount + convertedSample.value);
        }
      }
    }
  }

  return samplesDict;
}

function stringifySamples(samplesDict) {
  let samplesArray = [];
  for (let [bucket, value] of samplesDict.entries()) {
    // Stringify these so we can use assert_array_equals on them.
    samplesArray.push(bucket + ' => ' + value);
  }
  samplesArray.sort();
  return samplesArray;
}

function maybeDelay(delayParam) {
  if (delayParam) {
    return `&pipe=trickle(d${delayParam / 1000})`
  } else {
    return '';
  }
}

function createIgOverrides(nameAndBid, fragments, originOverride = null) {
  let originToUse = originOverride ? originOverride : MAIN_ORIGIN;
  return {
    name: nameAndBid,
    biddingLogicURL: createBiddingScriptURL({
                       origin: originToUse,
                       generateBid:
                           enableDebugMode + fragments.generateBidFragment,
                       reportWin: enableDebugMode + fragments.reportWinFragment,
                       bid: nameAndBid,
                       allowComponentAuction: true
                     }) +
        maybeDelay(fragments.bidderDelayFactor ?
                       fragments.bidderDelayFactor * nameAndBid :
                       null)
  };
}

function expectAndConsume(samplesDict, bucket, val) {
  assert_equals(samplesDict.get(bucket), val, 'sample in bucket ' + bucket);
  samplesDict.delete(bucket);
}

function createAuctionConfigOverrides(
    uuid, fragments, moreAuctionConfigOverrides = {}) {
  return {
    decisionLogicURL:
        createDecisionScriptURL(uuid, {
          origin: MAIN_ORIGIN,
          scoreAd: enableDebugMode + fragments.scoreAdFragment,
          reportResult: enableDebugMode + fragments.reportResultFragment
        }) +
        maybeDelay(fragments.sellerDelay),
    seller: MAIN_ORIGIN,
    interestGroupBuyers: [MAIN_ORIGIN],
    ...moreAuctionConfigOverrides
  };
}

// Runs an auction with numGroups interest groups, "1" and "2", etc., with
// fragments.generateBidFragment/fragments.reportWinFragment/
// fragments.scoreAdFragment/fragments.reportResultFragment
// expected to make some Private Aggregation contributions.
// Returns the collected samples.
async function runPrivateAggregationTest(
    test, uuid, fragments, numGroups = 2, moreAuctionConfigOverrides = {}) {
  await resetReports(MAIN_ORIGIN + MAIN_PATH);
  await resetReports(MAIN_ORIGIN + DEBUG_PATH);

  for (let i = 1; i <= numGroups; ++i) {
    await joinCrossOriginInterestGroup(
        test, uuid, MAIN_ORIGIN, createIgOverrides(i, fragments));
  }

  const auctionConfigOverrides =
      createAuctionConfigOverrides(uuid, fragments, moreAuctionConfigOverrides);

  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfigOverrides);
  return await getDebugSamples(DEBUG_PATH);
}

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const fragments = {
    generateBidFragment: `
      privateAggregation.contributeToHistogram({ bucket: 1n, value: 2 });`,

    reportWinFragment:
        `privateAggregation.contributeToHistogram({ bucket: 2n, value: 3 });`,

    scoreAdFragment:
        `privateAggregation.contributeToHistogram({ bucket: 3n, value: 4 });`,

    reportResultFragment:
        `privateAggregation.contributeToHistogram({ bucket: 4n, value: 5 });`
  };

  const samples = await runPrivateAggregationTest(test, uuid, fragments);
  assert_array_equals(
      stringifySamples(samples),
      [
        '1 => 4',  // doubled since it's reported twice.
        '2 => 3',
        '3 => 8',  // doubled since it's reported twice.
        '4 => 5'
      ]);
}, 'Basic contributions');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const fragments = {
    generateBidFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always',
          { bucket: 1n, value: 2 });`,

    reportWinFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always',
          { bucket: 2n, value: 3 });`,

    scoreAdFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always',
          { bucket: 3n, value: 4 });`,

    reportResultFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always',
          { bucket: 4n, value: 5 });`
  };

  const samples = await runPrivateAggregationTest(test, uuid, fragments);
  assert_array_equals(
      stringifySamples(samples),
      [
        '1 => 4',  // doubled since it's reported twice.
        '2 => 3',
        '3 => 8',  // doubled since it's reported twice.
        '4 => 5'
      ]);
}, 'reserved.always');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const fragments = {
    generateBidFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.win',
          { bucket: 1n, value: interestGroup.name });`,

    reportWinFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.win',
          { bucket: 2n, value: 3 });`,

    scoreAdFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.win',
          { bucket: 3n, value: bid });`,

    reportResultFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.win',
          { bucket: 4n, value: 5 });`
  };

  const samples = await runPrivateAggregationTest(test, uuid, fragments);
  assert_array_equals(
      stringifySamples(samples),
      [
        '1 => 2',  // winning IG name
        '2 => 3',
        '3 => 2',  // winning bid
        '4 => 5'
      ]);
}, 'reserved.win');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const fragments = {
    generateBidFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.loss',
          { bucket: 1n, value: interestGroup.name });`,

    reportWinFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.loss',
          { bucket: 2n, value: 3 });`,

    scoreAdFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.loss',
          { bucket: 3n, value: bid });`,

    reportResultFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.loss',
          { bucket: 4n, value: 5 });`
  };

  const samples = await runPrivateAggregationTest(test, uuid, fragments);

  // No reserved.loss from reporting since they only run for winners.
  assert_array_equals(
      stringifySamples(samples),
      [
        '1 => 1',  // losing IG name
        '3 => 1',  // losing bid
      ]);
}, 'reserved.loss');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const fragments = {
    generateBidFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once',
          { bucket: 1n, value: interestGroup.name });`,

    reportWinFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once',
          { bucket: 2n, value: 3 });`,

    scoreAdFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once',
          { bucket: 3n, value: bid });`,

    reportResultFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once',
          { bucket: 4n, value: 5 });`
  };

  const samples =
      stringifySamples(await runPrivateAggregationTest(test, uuid, fragments));

  // No reserved.once from reporting since it throws an exception.
  // bidder/scorer just pick one.
  assert_equals(samples.length, 2, 'samples array length');
  assert_in_array(samples[0], ['1 => 1', '1 => 2'], 'samples[0]');
  assert_in_array(samples[1], ['3 => 1', '3 => 2'], 'samples[1]');
}, 'reserved.once');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const fragments = {
    generateBidFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once',
          { bucket: 1n, value: 1 });`,

    reportWinFragment: `
      try {
        privateAggregation.contributeToHistogramOnEvent(
            'reserved.once',
            { bucket: 2n, value: 2 });
      } catch (e) {
        privateAggregation.contributeToHistogramOnEvent(
            'reserved.always',
            { bucket: 2n, value: (e instanceof TypeError ? 3 : 4) });
      }`,

    scoreAdFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once',
          { bucket: 3n, value: 4 });`,

    reportResultFragment: `
      try {
        privateAggregation.contributeToHistogramOnEvent(
            'reserved.once',
            { bucket: 4n, value: 5 });
      } catch (e) {
        privateAggregation.contributeToHistogramOnEvent(
            'reserved.always',
            { bucket: 4n, value: (e instanceof TypeError ? 6 : 7) });
      }`
  };

  const samples =
      stringifySamples(await runPrivateAggregationTest(test, uuid, fragments));

  assert_array_equals(samples, [
    '1 => 1',
    '2 => 3',
    '3 => 4',
    '4 => 6',
  ]);
}, 'no reserved.once in reporting');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await resetReports(ALT_ORIGIN + DEBUG_PATH);
  await resetReports(ALT_ORIGIN + MAIN_PATH);

  const fragments = {
    generateBidFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once', {
              bucket: {baseValue: 'average-code-fetch-time', offset: 0n},
              value: 1});`,

    reportWinFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always', {
              bucket: {baseValue: 'average-code-fetch-time', offset: 100000n},
              value: 1});`,

    bidderDelayFactor: 200,

    scoreAdFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once', {
              bucket: {baseValue: 'average-code-fetch-time', offset: 200000n},
              value: 1});`,

    reportResultFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always', {
              bucket: {baseValue: 'average-code-fetch-time', offset: 300000n},
              value: 1});`,

    sellerDelay: 500
  };

  const altFragments = {
    generateBidFragment: fragments.generateBidFragment,
    bidderDelayFactor: 1000
  };

  await joinCrossOriginInterestGroup(
      test, uuid, ALT_ORIGIN, createIgOverrides('1', altFragments, ALT_ORIGIN));
  const auctionConfigOverrides = {
    interestGroupBuyers: [MAIN_ORIGIN, ALT_ORIGIN]
  };

  const samples = await runPrivateAggregationTest(
      test, uuid, fragments, 3, auctionConfigOverrides);

  let generateBidVal = -1;
  let reportWinVal = -1;
  let scoreAdVal = -1;
  let reportResultVal = -1;
  assert_equals(samples.size, 4, 'main domain samples');

  for (let [bucket, val] of samples.entries()) {
    assert_equals(val, 1n, 'bucket val');
    if (0n <= bucket && bucket < 100000n) {
      generateBidVal = Number(bucket - 0n);
    } else if (100000n <= bucket && bucket < 200000n) {
      reportWinVal = Number(bucket - 100000n);
    } else if (200000n <= bucket && bucket < 300000n) {
      scoreAdVal = Number(bucket - 200000n);
    } else if (300000n <= bucket && bucket < 400000n) {
      reportResultVal = Number(bucket - 300000n);
    } else {
      assert_unreached('Unexpected bucket number ' + bucket);
    }
  }

  assert_greater_than_equal(generateBidVal, 400, 'generateBid code fetch time');
  assert_greater_than_equal(reportWinVal, 600, 'reportWin code fetch time');
  assert_greater_than_equal(scoreAdVal, 500, 'scoreAd code fetch time');
  assert_greater_than_equal(
      reportResultVal, 500, 'reportResult code fetch time');

  let otherSamples = await getDebugSamples(ALT_ORIGIN + DEBUG_PATH);
  assert_equals(otherSamples.size, 1, 'alt domain samples');
  let otherGenerateBidVal = -1;
  for (let [bucket, val] of otherSamples.entries()) {
    assert_equals(val, 1n, 'other bucket val');
    if (0n <= bucket && bucket < 100000n) {
      otherGenerateBidVal = Number(bucket - 0n);
    } else {
      assert_unreached('Unexpected other bucket number ' + bucket);
    }
  }
  assert_greater_than_equal(
      otherGenerateBidVal, 1000, 'other generateBid code fetch time');
}, 'average-code-fetch-time');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const fragments = {
    generateBidFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once', {
              bucket: {baseValue: 'percent-scripts-timeout', offset: 0n},
              value: 1});
      if (interestGroup.name === '1') {
        while (true) {}
      }
      `,

    reportWinFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always', {
              bucket: {baseValue: 'percent-scripts-timeout', offset: 200n},
              value: 1});
      while(true) {}`,

    scoreAdFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once', {
              bucket: {baseValue: 'percent-scripts-timeout', offset: 400n},
              value: 1});
      if (bid == 2) {
        while (true) {}
      }
      `,

    reportResultFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always', {
              bucket: {baseValue: 'percent-scripts-timeout', offset: 600n},
              value: 1});`
  };

  const samples = await runPrivateAggregationTest(test, uuid, fragments, 3);

  let expected = [
    '33 => 1',   // 33% of generateBid  (base bucket 0)
    '300 => 1',  // 100% of reportWin   (base bucket 200)
    '450 => 1',  // 50% of scoreAd      (base bucket 400)
    '600 => 1',  // 0% of reportResult  (base bucket 600)
  ].sort();

  assert_array_equals(stringifySamples(samples), expected);
}, 'percent-scripts-timeout');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await resetReports(ALT_ORIGIN + DEBUG_PATH);
  await resetReports(ALT_ORIGIN + MAIN_PATH);

  const ADDITIONAL_BID_PUBLIC_KEY =
      '11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo=';

  // Join a negative group, one without ads.
  // These shouldn't count towards participant number.
  await joinNegativeInterestGroup(
      test, MAIN_ORIGIN, 'some negative group', ADDITIONAL_BID_PUBLIC_KEY);
  await joinCrossOriginInterestGroup(test, uuid, MAIN_ORIGIN, {ads: []});

  const fragments = {
    generateBidFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once', {
              bucket: {baseValue: 'participating-ig-count', offset: 0n},
              value: 1});`,

    reportWinFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always', {
              bucket: {baseValue: 'participating-ig-count', offset: 200n},
              value: 1});`,

    scoreAdFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once', {
              bucket: {baseValue: 'participating-ig-count', offset: 400n},
              value: 1});`,

    reportResultFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always', {
              bucket: {baseValue: 'participating-ig-count', offset: 600n},
              value: 1});`
  };

  // ... and a different participant should get their own samples.
  await joinCrossOriginInterestGroup(
      test, uuid, ALT_ORIGIN, createIgOverrides('1', fragments, ALT_ORIGIN));
  await joinCrossOriginInterestGroup(
      test, uuid, ALT_ORIGIN, createIgOverrides('2', fragments, ALT_ORIGIN));
  const auctionConfigOverrides = {
    interestGroupBuyers: [MAIN_ORIGIN, ALT_ORIGIN]
  };

  const samples = await runPrivateAggregationTest(
      test, uuid, fragments, 5, auctionConfigOverrides);

  let expected = [
    '5 => 1',    // 5 in generateBid  (base bucket 0)
    '205 => 1',  // 5 in reportWin    (base bucket 200)
    '400 => 1',  // 0 in scoreAd      (base bucket 400)
    '600 => 1',  // 0 in reportResult (base bucket 600)
  ].sort();

  assert_array_equals(stringifySamples(samples), expected);

  let otherSamples = await getDebugSamples(ALT_ORIGIN + DEBUG_PATH);
  assert_array_equals(stringifySamples(otherSamples), ['2 => 1']);
}, 'participating-ig-count');


subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const fragments = {
    generateBidFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once', {
              bucket: {
                  baseValue: 'percent-igs-cumulative-timeout',
                  offset: 0n
              },
              value: 1});
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once', {
              bucket: {
                  baseValue: 'cumulative-buyer-time',
                  offset: 10000n
              },
              value: 1});
      setBid({bid: interestGroup.name, render: interestGroup.ads[0].renderURL});
      while (true) {}
      `,

    reportWinFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always', {
              bucket: {
                  baseValue: 'percent-igs-cumulative-timeout',
                  offset: 200n
              },
              value: 1});
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always', {
              bucket: {
                  baseValue: 'cumulative-buyer-time',
                  offset: 20000n
              },
              value: 1});
      `,

    scoreAdFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once', {
              bucket: {
                  baseValue: 'percent-igs-cumulative-timeout',
                  offset: 400n
              },
              value: 1});
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once', {
              bucket: {
                  baseValue: 'cumulative-buyer-time',
                  offset: 40000n
              },
              value: 1});
      `,

    reportResultFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always', {
              bucket: {
                  baseValue: 'percent-igs-cumulative-timeout',
                  offset: 600n
              },
              value: 1});
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always', {
              bucket: {
                  baseValue: 'cumulative-buyer-time',
                  offset: 60000n
              },
              value: 1});`
  };

  const auctionConfigOverrides = {
    perBuyerTimeouts: {
      '*': 500  // max.
    },
    perBuyerCumulativeTimeouts: {'*': 2000}
  };

  const samples = await runPrivateAggregationTest(
      test, uuid, fragments, 15, auctionConfigOverrides);

  // Timeout is reported as 3000 (limit + 1000) for generateBid
  // and reportWin, as 0 for the seller methods.
  expectAndConsume(samples, 13000n, 1n);  // base is 10,000
  expectAndConsume(samples, 23000n, 1n);
  expectAndConsume(samples, 40000n, 1n);
  expectAndConsume(samples, 60000n, 1n);

  // percent time is 0 on the seller side.
  expectAndConsume(samples, 400n, 1n);
  expectAndConsume(samples, 600n, 1n);

  assert_equals(samples.size, 2, 'buyer samples');

  let percentGenerateBid = -1;
  let percentReportWin = -1;

  for (let [bucket, val] of samples.entries()) {
    assert_equals(val, 1n, 'bucket val');
    if (0n <= bucket && bucket <= 110n) {
      percentGenerateBid = bucket;
    } else if (200n <= bucket && bucket <= 310n) {
      percentReportWin = bucket - 200n;
    } else {
      assert_unreached('Unexpected bucket number ' + bucket);
    }
  }

  assert_equals(
      percentGenerateBid, percentReportWin,
      'same % in generateBid and reportWin');

  // This assumes that at least some time out; which may not be guaranteed with
  // sufficient level of parallelism. At any rate, the denominator is 15,
  // however, so only some percentages are possible.
  assert_in_array(
      percentGenerateBid,
      [6n, 13n, 20n, 26n, 33n, 40n, 46n, 53n, 60n, 66n, 73n, 80n, 86n, 93n],
      'percent timeout is as expected');
}, 'percent-igs-cumulative-timeout, and cumulative-buyer-time when hit');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const fragments = {
    generateBidFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once', {
              bucket: {baseValue: 'cumulative-buyer-time', offset: 0n},
              value: 1});`,

    reportWinFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always', {
              bucket: {baseValue: 'cumulative-buyer-time', offset: 200n},
              value: 1});`,

    scoreAdFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once', {
              bucket: {baseValue: 'cumulative-buyer-time', offset: 400n},
              value: 1});`,

    reportResultFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always', {
              bucket: {baseValue: 'cumulative-buyer-time', offset: 600n},
              value: 1});`
  };

  const samples = await runPrivateAggregationTest(test, uuid, fragments, 5);

  // 0s for all the bases.
  let expected = ['0 => 1', '200 => 1', '400 => 1', '600 => 1'].sort();

  assert_array_equals(stringifySamples(samples), expected);
}, 'cumulative-buyer-time when not configured');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const fragments = {
    generateBidFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once', {
              bucket: {baseValue: 'cumulative-buyer-time', offset: 0n},
              value: 1});`,

    reportWinFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always', {
              bucket: {baseValue: 'cumulative-buyer-time', offset: 10000n},
              value: 1});`,

    scoreAdFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once', {
              bucket: {baseValue: 'cumulative-buyer-time', offset: 20000n},
              value: 1});`,

    reportResultFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always', {
              bucket: {baseValue: 'cumulative-buyer-time', offset: 30000n},
              value: 1});`
  };

  const auctionConfigOverrides = {perBuyerCumulativeTimeouts: {'*': 4000}};

  const samples = await runPrivateAggregationTest(
      test, uuid, fragments, 5, auctionConfigOverrides);

  // Sellers stuff is just 0s (so 1 to the base bucket offset).
  expectAndConsume(samples, 20000n, 1n);
  expectAndConsume(samples, 30000n, 1n);

  assert_equals(samples.size, 2, 'buyer samples');

  let timeGenerateBid = -1;
  let timeReportWin = -1;

  for (let [bucket, val] of samples.entries()) {
    assert_equals(val, 1n, 'bucket val');
    if (0n <= bucket && bucket <= 5000n) {
      timeGenerateBid = bucket;
    } else if (10000n <= bucket && bucket <= 15000n) {
      timeReportWin = bucket - 10000n;
    } else {
      assert_unreached('Unexpected bucket number');
    }
  }

  assert_equals(
      timeGenerateBid, timeReportWin, 'same time in generateBid and reportWin');

  // This assume this takes more than 0ms to run; it's not really required to
  // be the case, but feels like a realistic assumption that makes the test
  // more useful.
  assert_true(
      1n <= timeGenerateBid && timeGenerateBid <= 4000n,
      'time ' + timeGenerateBid + ' is reasonable and non-zero');
}, 'cumulative-buyer-time when configured');


async function testStorageQuotaMetric(test, name) {
  const uuid = generateUuid(test);
  const fragments = {
    generateBidFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once', {
              bucket: {baseValue: '${name}', offset: 0n},
              value: 1});`,

    reportWinFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always', {
              bucket: {baseValue: '${name}', offset: 10000n},
              value: 1});`,

    scoreAdFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once', {
              bucket: {baseValue: '${name}', offset: 20000n},
              value: 1});`,

    reportResultFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always', {
              bucket: {baseValue: '${name}', offset: 30000n},
              value: 1});`
  };

  const samples = await runPrivateAggregationTest(test, uuid, fragments, 5);

  // Sellers stuff is just 0s (so 1 to the base bucket offset).
  expectAndConsume(samples, 20000n, 1n);
  expectAndConsume(samples, 30000n, 1n);

  assert_equals(samples.size, 2, 'buyer samples');

  let generateBidVal = -1;
  let reportWinVal = -1;

  for (let [bucket, val] of samples.entries()) {
    assert_equals(val, 1n, 'bucket val');
    if (0n <= bucket && bucket < 10000n) {
      generateBidVal = Number(bucket);
    } else if (10000n <= bucket && bucket <= 20000n) {
      reportWinVal = Number(bucket - 10000n);
    } else {
      assert_unreached('Unexpected bucket number ' + bucket);
    }
  }

  assert_equals(
      generateBidVal, reportWinVal, 'same value in generateBid and reportWin');

  // We don't know what the impls quota is, or even how much we are using,
  // but at least make sure it's in range.
  assert_between_inclusive(
      generateBidVal, 0, 110, 'reported percent value is in expected range');
}

subsetTest(promise_test, async test => {
  await testStorageQuotaMetric(test, 'percent-regular-ig-count-quota-used');
}, 'percent-regular-ig-count-quota-used');

subsetTest(promise_test, async test => {
  await testStorageQuotaMetric(test, 'percent-negative-ig-count-quota-used');
}, 'percent-negative-ig-count-quota-used');

subsetTest(promise_test, async test => {
  await testStorageQuotaMetric(test, 'percent-ig-storage-quota-used');
}, 'percent-ig-storage-quota-used');


async function testStorageUsageMetric(test, name, min) {
  const uuid = generateUuid(test);
  const spacing = 1000000000n;
  const fragments = {
    generateBidFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once', {
              bucket: {baseValue: '${name}', offset: 0n},
              value: 1});`,

    reportWinFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always', {
              bucket: {baseValue: '${name}', offset: ${spacing}n},
              value: 1});`,

    scoreAdFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once', {
              bucket: {baseValue: '${name}', offset: 2n * ${spacing}n},
              value: 1});`,

    reportResultFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always', {
              bucket: {baseValue: '${name}', offset: 3n * ${spacing}n},
              value: 1});`
  };

  await joinNegativeInterestGroup(
      test, MAIN_ORIGIN, 'some negative group', ADDITIONAL_BID_PUBLIC_KEY);
  await joinNegativeInterestGroup(
      test, MAIN_ORIGIN, 'some negative group 2', ADDITIONAL_BID_PUBLIC_KEY);
  await joinCrossOriginInterestGroup(
      test, uuid, MAIN_ORIGIN,
      {ads: [], name: 'Big group w/o ads'.padEnd(50000)});

  const samples = await runPrivateAggregationTest(test, uuid, fragments, 5);

  // Sellers stuff is just 0s (so 1 to the base bucket offset).
  expectAndConsume(samples, 2n * spacing, 1n);
  expectAndConsume(samples, 3n * spacing, 1n);

  assert_equals(samples.size, 2, 'buyer samples');

  let generateBidVal = -1;
  let reportWinVal = -1;

  for (let [bucket, val] of samples.entries()) {
    assert_equals(val, 1n, 'bucket val');
    if (0n <= bucket && bucket < spacing) {
      generateBidVal = bucket;
    } else if (spacing <= bucket && bucket < 2n * spacing) {
      reportWinVal = bucket - spacing;
    } else {
      assert_unreached('Unexpected bucket number ' + bucket);
    }
  }

  assert_equals(
      generateBidVal, reportWinVal, 'same value in generateBid and reportWin');

  assert_true(
      generateBidVal >= BigInt(min),
      'reported value should be at least ' + min + ' but is ' + generateBidVal);
}

subsetTest(promise_test, async test => {
  // 5 regular Igs + one ad less.
  await testStorageUsageMetric(test, 'regular-igs-count', 6);
}, 'regular-igs-count');

subsetTest(promise_test, async test => {
  // 2 negative IGs
  await testStorageUsageMetric(test, 'negative-igs-count', 2);
}, 'negative-igs-count');

subsetTest(promise_test, async test => {
  // The big group has a 50,000 character name
  await testStorageUsageMetric(test, 'ig-storage-used', 50000);
}, 'ig-storage-used');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await resetReports(MAIN_ORIGIN + MAIN_PATH);
  await resetReports(MAIN_ORIGIN + DEBUG_PATH);
  await resetReports(ALT_ORIGIN + MAIN_PATH);
  await resetReports(ALT_ORIGIN + DEBUG_PATH);

  const fragments = {
    generateBidFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once',
          { bucket: 1n, value: 2 });`,

    reportWinFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always',
          { bucket: 2n, value: 3 });`,

    scoreAdFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once',
          { bucket: 3n, value: 4 });`,

    reportResultFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always',
          { bucket: 4n, value: 5 });`
  };

  // 4 IGs in main origin, 2 in alt origin.
  for (let i = 1; i <= 4; ++i) {
    await joinCrossOriginInterestGroup(
        test, uuid, MAIN_ORIGIN, createIgOverrides(i, fragments));
  }

  for (let i = 1; i <= 2; ++i) {
    await joinCrossOriginInterestGroup(
        test, uuid, ALT_ORIGIN, createIgOverrides(i, fragments, ALT_ORIGIN));
  }

  // Both groups in component auction 1, only alt group in component auction 2.
  const subAuction1 = createAuctionConfigOverrides(
      uuid, fragments, {interestGroupBuyers: [MAIN_ORIGIN, ALT_ORIGIN]});
  const subAuction2 = createAuctionConfigOverrides(
      uuid, fragments, {interestGroupBuyers: [ALT_ORIGIN]});

  const topFragments = {
    scoreAdFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.once',
          { bucket: 5n, value: 6 });`,

    reportResultFragment: `
      privateAggregation.contributeToHistogramOnEvent(
          'reserved.always',
          { bucket: 6n, value: 7 });`
  };
  const mainAuction = createAuctionConfigOverrides(
      uuid, topFragments,
      {interestGroupBuyers: [], componentAuctions: [subAuction1, subAuction2]});

  await runBasicFledgeAuctionAndNavigate(test, uuid, mainAuction);
  let samples = await getDebugSamples(DEBUG_PATH);
  let otherSamples = await getDebugSamples(ALT_ORIGIN + DEBUG_PATH);
  let expected = [
    '1 => 2',  // generateBid only in first component, so happens 1.
    '2 => 3',  // reportWin once.
    '3 => 8',  // Once per each component auction (out of total 6 scored).
    '4 => 5',  // component reportResult once.
    '5 => 6',  // top-level scoreAd once.
    '6 => 7',  // top-level reportResult.
  ].sort();
  let otherExpected = [
    '1 => 4',  // generateBid in each components, so twice, out of 4 executions.
  ].sort();
  assert_array_equals(stringifySamples(samples), expected);
  assert_array_equals(stringifySamples(otherSamples), otherExpected);
}, 'report.once in a component auction');
