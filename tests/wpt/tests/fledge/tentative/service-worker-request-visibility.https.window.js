// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-last

"use strict;"

const SERVICE_WORKER_SCRIPT = "resources/service-worker-helper.js";

// List of URL fragments that uniquely identify private requests.
// These are used to detect if a service worker has inadvertently
// accessed any of these private requests, which should not be visible to it.
const PRIVATE_REQUEST_FILE_NAMES = [
  'trusted-bidding-signals.py',
  'update-url.py', // This requires another test, since it takes a while,
  // see TODO file for more info.
  'wasm-helper.py',
  'bidding-logic.py',
  'decision-logic.py',
  'trusted-scoring-signals.py',
  'trusted-bidding-signals.py',
  'bidder_report',
  'seller_report'
];

// List of URL fragments that uniquely identify public requests.
// These are used to verify that a service worker can correctly
// access and intercept these public requests as expected.
const PUBLIC_REQUEST_FILE_NAMES = [
  'direct-from-seller-signals.py',
];

const COMPLETE_TEST_URL = 'complete-test'

const CURRENT_SCOPE = "/fledge/tentative/"

async function registerAndActivateServiceWorker(test) {
  // Unregister existing service worker (if any)
  const existingRegistration = await navigator.serviceWorker.getRegistration(CURRENT_SCOPE);
  if (existingRegistration) {
    await existingRegistration.unregister();
  }

  // Register new service worker
  var newRegistration = await navigator.serviceWorker.register(`./${SERVICE_WORKER_SCRIPT}`, { scope: CURRENT_SCOPE });

  test.add_cleanup(async () => {
    await newRegistration.unregister();
  });

  await navigator.serviceWorker.ready;

  // Wait for the page to be controlled by the service worker.
  // This is needed as navigator.serviceWorker.ready does not
  // guarantee that the page is being controlled.
  // See https://github.com/slightlyoff/ServiceWorker/issues/799.
  await new Promise(resolve => {
    if (navigator.serviceWorker.controller) {
      resolve();
    } else {
      navigator.serviceWorker.addEventListener('controllerchange', resolve);
    }
  });

  // Validate the service worker
  if (!navigator.serviceWorker.controller.scriptURL.includes(SERVICE_WORKER_SCRIPT)) {
    throw new Error('Failed to register service worker');
  }
}

async function setUpServiceWorkerAndGetBroadcastChannel(test) {
  await registerAndActivateServiceWorker(test);
  return new BroadcastChannel("requests-test");
}

// Waits for a service worker to observe specific URL filenames via a BroadcastChannel.
// Resolves when all expected URL filenames are seen.
function awaitServiceWorkerURLPromise(broadcastChannel, expectedURLFileNames,
  unexpectedURLFileNames) {
  const seenURLs = new Set();
  return new Promise((resolve, reject) => {
    broadcastChannel.addEventListener('message', (event) => {
      var url = event.data.url;
      var fileName = url.substring(url.lastIndexOf('/') + 1);
      if (expectedURLFileNames.includes(fileName)) {
        seenURLs.add(fileName);
      }
      if (unexpectedURLFileNames.includes(fileName)) {
        reject(`unexpected result: ${fileName}`);
      }
      // Resolve when all `expectedURLs` have been seen.
      if (seenURLs.size === expectedURLFileNames.length) {
        resolve();
      }
    });
  });
}

// Tests that public requests are seen by the service worker.
// Specifically anything that contains:
// - 'direct-from-seller-signals.py'

// This test works by having the service worker send a message over
// the broadcastChannel, if it sees a request that contains any of
// the following strings above, it will send a 'passed' result and
// also change the variable 'finish_test', to true, so that guarantees
// that the request was seen before we complete the test.
subsetTest(promise_test, async test => {
  const broadcastChannel = await setUpServiceWorkerAndGetBroadcastChannel(test);
  let finishTest = awaitServiceWorkerURLPromise(
    broadcastChannel,
    PUBLIC_REQUEST_FILE_NAMES,
    PRIVATE_REQUEST_FILE_NAMES);

  await fetchDirectFromSellerSignals({ 'Buyer-Origin': window.location.origin });
  await finishTest;
}, "Make sure service workers do see public requests.");

// Tests that private requests are not seen by the service worker.
// Specifically anything that contains:
// - 'resources/trusted-bidding-signals.py'
// - 'resources/trusted-scoring-signals.py'
// - 'wasm-helper.py'
// - 'bidding-logic.py'
// - 'decision-logic.py'
// - 'seller_report'
// - 'bidder_report'

// This test works by having the service worker send a message
// over the broadcastChannel, if it sees a request that contains
// any of the following strings above, it will send a 'failed'
// result which will cause assert_false case to fail.
subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const broadcastChannel = await setUpServiceWorkerAndGetBroadcastChannel(test);

  let finishTest = awaitServiceWorkerURLPromise(
    broadcastChannel,
    /*expectedURLFileNames=*/[COMPLETE_TEST_URL],
    PRIVATE_REQUEST_FILE_NAMES)

  let interestGroupOverrides = {
    biddingWasmHelperURL: `${RESOURCE_PATH}wasm-helper.py`,
    trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL,
    trustedScoringSignalsURL: TRUSTED_SCORING_SIGNALS_URL,
  };

  await joinInterestGroup(test, uuid, interestGroupOverrides);
  await runBasicFledgeAuctionAndNavigate(test, uuid);
  // By verifying that these requests are observed we can assume
  // none of the other requests were seen by the service-worker.
  await waitForObservedRequests(
    uuid,
    [createBidderReportURL(uuid), createSellerReportURL(uuid)]);

  // We use this fetch to complete the test.
  await fetch(COMPLETE_TEST_URL);
  await finishTest;
}, "Make sure service workers do not see private requests");
