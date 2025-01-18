// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=resources/ba-fledge-util.sub.js
// META: script=resources/fledge-util.sub.js
// META: script=third_party/cbor-js/cbor.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-6
// META: variant=?7-last

// These tests focus on the paggResponse field in AuctionConfig's
// serverResponse, i.e. auctions involving private aggregation reporting. NOTE:
// Due to debug mode being disabled for B&A's Private Aggregation reports, these
// tests just exercise the code paths and ensure that correct number of reports
// are sent -- they don't otherwise verify report content.

// Runs responseMutator on a minimal correct server response, and expects
// either success/failure based on expectWin.
// Copied from server-response.https.window.js.
// TODO(qingxinwu): move to a shared utility file.
async function testWithMutatedServerResponse(
    test, expectWin, responseMutator, igMutator = undefined,
    ownerOverride = null) {
  let finalIgOwner = ownerOverride ? ownerOverride : window.location.origin;
  const uuid = generateUuid(test);
  const adA = createTrackerURL(finalIgOwner, uuid, 'track_get', 'a');
  const adB = createTrackerURL(finalIgOwner, uuid, 'track_get', 'b');
  const adsArray =
      [{renderURL: adA, adRenderId: 'a'}, {renderURL: adB, adRenderId: 'b'}];
  let ig = {ads: adsArray};
  if (igMutator) {
    igMutator(ig, uuid);
  }
  if (ownerOverride !== null) {
    await joinCrossOriginInterestGroup(test, uuid, ownerOverride, ig);
  } else {
    await joinInterestGroup(test, uuid, ig);
  }

  const result = await navigator.getInterestGroupAdAuctionData({
    coordinatorOrigin: await BA.configureCoordinator(),
    seller: window.location.origin
  });
  assert_true(result.requestId !== null);
  assert_true(result.request.length > 0);

  let decoded = await BA.decodeInterestGroupData(result.request);

  let serverResponseMsg = {
    'biddingGroups': {},
    'adRenderURL': ig.ads[0].renderURL,
    'interestGroupName': DEFAULT_INTEREST_GROUP_NAME,
    'interestGroupOwner': finalIgOwner,
  };
  serverResponseMsg.biddingGroups[finalIgOwner] = [0];
  await responseMutator(serverResponseMsg, uuid);

  let serverResponse =
      await BA.encodeServerResponse(serverResponseMsg, decoded);

  let hashString = await BA.payloadHash(serverResponse);
  await BA.authorizeServerResponseHashes([hashString]);

  let auctionResult = await navigator.runAdAuction({
    'seller': window.location.origin,
    'interestGroupBuyers': [finalIgOwner],
    'requestId': result.requestId,
    'serverResponse': serverResponse,
    'resolveToConfig': true,
  });
  if (expectWin) {
    expectSuccess(auctionResult);
    return auctionResult;
  } else {
    expectNoWinner(auctionResult);
  }
}

// To better isolate from private aggregation tests run in parallel,
// don't use the usual origin here.
const MAIN_ORIGIN = OTHER_ORIGIN1;
const MAIN_PATH = '/.well-known/private-aggregation/report-protected-audience';

const BUCKET_ONE = new Uint8Array([
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x01
]);

function BigEndianInteger128ToUint8Array(val) {
  let buffer = new Uint8Array(16);
  for (let i = 15; i >= 0; i--) {
    buffer[i] = Number(val & 0xFFn);
    val >>= 8n;
  }
  return buffer;
}

function createSimplePerOriginPAggResponse(
    reportingOrigin = MAIN_ORIGIN, igIndex = 0, event = 'reserved.win',
    bucket = BUCKET_ONE, value = 10, filteringId = null) {
  let contribution = {};
  if (bucket !== null) {
    contribution.bucket = bucket;
  }
  if (value !== null) {
    contribution.value = value;
  }
  if (filteringId !== null) {
    contribution.filteringId = filteringId;
  }
  return {
    'reportingOrigin': reportingOrigin,
    'igContributions': [{
      'igIndex': igIndex,
      'eventContributions': [{'event': event, 'contributions': [contribution]}]
    }]
  };
}

async function privateAggregationTestWithMutatedServerResponse(
    test, expectWin, paggResponse, timeout = 5000 /*ms*/,
    ownerOverride = null) {
  await resetReports(MAIN_ORIGIN + MAIN_PATH);
  let result = await testWithMutatedServerResponse(
      test, expectWin,
      (msg, uuid) => {
        msg.paggResponse = paggResponse;
      },
      (ig, uuid) => {
        ig.ads[0].renderURL = createRenderURL(uuid);
      },
      ownerOverride);
  createAndNavigateFencedFrame(test, result);
  const reports = await pollReports(MAIN_PATH, timeout);
  return reports;
}

async function testInvalidPAggResponseFields(
    test, reportingOrigin = MAIN_ORIGIN, igIndex = 0, event = 'reserved.win',
    bucket = '1', value = 10, filteringId = null) {
  const paggResponse = [createSimplePerOriginPAggResponse(
      reportingOrigin, igIndex, event, bucket, value, filteringId)];

  let reports = await privateAggregationTestWithMutatedServerResponse(
      test,
      /*expectWin=*/ true, paggResponse, /*timeout=*/ 5000, MAIN_ORIGIN);
  assert_equals(reports, null);
}

// The next few methods are modified from Chrome-specific
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

/**
 * Verifies that a report's aggregation_service_payloads has the expected
 * fields. Currently for B&A's PAgg reports, debug mode is disabled, so we
 * cannot check contributions in payload.
 */
const verifyAggregationServicePayloads = (aggregation_service_payloads) => {
  assert_equals(aggregation_service_payloads.length, 1);
  const payload_obj = aggregation_service_payloads[0];

  assert_own_property(payload_obj, 'key_id');
  assert_own_property(payload_obj, 'payload');
  // Check the payload is base64 encoded. We do not decrypt the payload to
  // test its contents.
  atob(payload_obj.payload);

  // Check there are no extra keys
  assert_equals(Object.keys(payload_obj).length, expected_payload ? 3 : 2);
};

/**
 * Verifies that a report has the expected fields. The `expected_payload` should
 * be undefined.
 */
const verifyReport = (report, reporting_origin) => {
  assert_own_property(report, 'shared_info');
  let shared_info = JSON.parse(report.shared_info);
  assert_own_property(shared_info, 'reporting_origin');
  assert_equals(shared_info.reporting_origin, reporting_origin);
  assert_own_property(report, 'aggregation_service_payloads');
  assert_own_property(report, 'aggregation_coordinator_origin');
  // TODO(qingxinwu): Maybe add tests for coordinator origin.

  assert_not_own_property(report, 'debug_key');

  // Check there are no extra keys
  let expected_length = 3;
  assert_equals(Object.keys(report).length, expected_length);
};

subsetTest(promise_test, async test => {
  await testInvalidPAggResponseFields(test, 'http://non-https.com');
}, 'Private aggregation - invalid reporting origin');

subsetTest(
    promise_test,
    async test => {await testInvalidPAggResponseFields(test, MAIN_ORIGIN, 100)},
    'Private aggregation - invalid index');

subsetTest(promise_test, async test => {
  await testInvalidPAggResponseFields(
      test, MAIN_ORIGIN, 0, 'reserved.not-supported');
}, 'Private aggregation - invalid event');

subsetTest(promise_test, async test => {
  await testInvalidPAggResponseFields(
      test, MAIN_ORIGIN, 0, 'reserved.win', /*bucket=*/ null);
}, 'Private aggregation - missing required bucket');

subsetTest(promise_test, async test => {
  await testInvalidPAggResponseFields(
      test, MAIN_ORIGIN, 0, 'reserved.win', new Uint8Array([
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x01
      ]));
}, 'Private aggregation - bucket is bigger than 128 bits');

subsetTest(promise_test, async test => {
  await testInvalidPAggResponseFields(
      test, MAIN_ORIGIN, 0, 'reserved.win', BUCKET_ONE, /*value=*/ null);
}, 'Private aggregation - missing required value');

subsetTest(promise_test, async test => {
  await testInvalidPAggResponseFields(
      test, MAIN_ORIGIN, 0, 'reserved.win', BUCKET_ONE, 10, 10000);
}, 'Private aggregation - invalid filteringId');

subsetTest(promise_test, async test => {
  const paggResponse = [createSimplePerOriginPAggResponse()];

  let reports = await privateAggregationTestWithMutatedServerResponse(
      test,
      /*expectWin=*/ true, paggResponse, /*timeout=*/ 6000, MAIN_ORIGIN);
  assert_equals(reports.length, 1);
  let report = JSON.parse(reports[0]);
  verifyReport(report, MAIN_ORIGIN);
}, 'Private aggregation - successfully sent report');

// TODO(qingxinwu): may add a test for custom event type if possible.

subsetTest(promise_test, async test => {
  const paggResponse = [{
    'reportingOrigin': MAIN_ORIGIN,
    'igContributions': [{
      'igIndex': 0,
      'eventContributions': [
        {
          'event': 'reserved.win',
          'contributions': [{'value': 10}, {'bucket': BUCKET_ONE, 'value': 11}]
        },
        {
          'event': 'reserved.not-supported',
          'contributions':
              [{'bucket': BigEndianInteger128ToUint8Array(2n), 'value': 22}]
        },
      ]
    }]
  }];

  let reports = await privateAggregationTestWithMutatedServerResponse(
      test,
      /*expectWin=*/ true, paggResponse, /*timeout=*/ 6000, MAIN_ORIGIN);
  assert_equals(reports.length, 1);
  let report = JSON.parse(reports[0]);
  verifyReport(report, MAIN_ORIGIN);
}, 'Private aggregation - invalid contributions do not affect valid ones');

// TODO(qingxinwu): privateAggregation multi-seller.
