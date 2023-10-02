// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-4
// META: variant=?5-8
// META: variant=?9-12
// META: variant=?13-last

"use strict;"

const OTHER_ORIGIN1 = 'https://{{hosts[alt][]}}:{{ports[https][0]}}';
const OTHER_ORIGIN2 = 'https://{{hosts[alt][]}}:{{ports[https][1]}}';

// Runs "script" in "iframe" via an eval call. The iframe must have been
// created by calling "createIframe()" below. "param" is passed to the
// context "script" is run in, so can be used to pass objects that
// "script" references that can't be serialized to a string, like
// fencedFrameConfigs.
async function runInIframe(test, iframe, script, param) {
  const messageUuid = generateUuid(test);
  let receivedResponse = {};

  let promise = new Promise(function(resolve, reject) {
    function WaitForMessage(event) {
      if (event.data.messageUuid != messageUuid)
        return;
      receivedResponse = event.data;
      if (event.data.result === 'success') {
        resolve();
      } else {
        reject(event.data.result);
      }
    }
    window.addEventListener('message', WaitForMessage);
    iframe.contentWindow.postMessage(
        {messageUuid: messageUuid, script: script, param: param}, '*');
  });
  await promise;
  return receivedResponse.returnValue;
}

// Creates an iframe and navigates to a URL on "origin", and waits for the URL
// to finish loading by waiting for the iframe to send an event. Then returns
// the iframe.
//
// Also adds a cleanup callback to "test", which runs all cleanup functions
// added to the iframe and waits for them to complete, and then destroys the
// iframe.
async function createIframe(test, origin, permissions) {
  const iframeUuid = generateUuid(test);
  const iframeUrl = `${origin}${RESOURCE_PATH}iframe.sub.html?uuid=${iframeUuid}`;
  let iframe = document.createElement('iframe');
  await new Promise(function(resolve, reject) {
    function WaitForMessage(event) {
      if (event.data.messageUuid != iframeUuid)
        return;
      if (event.data.result === 'load complete') {
        resolve();
      } else {
        reject(event.data.result);
      }
    }
    window.addEventListener('message', WaitForMessage);
    if (permissions)
      iframe.allow = permissions;
    iframe.src = iframeUrl;
    document.body.appendChild(iframe);

    test.add_cleanup(async () => {
      await runInIframe(test, iframe, "await test_instance.do_cleanup();");
      document.body.removeChild(iframe);
    });
  });
  return iframe;
}

////////////////////////////////////////////////////////////////////////////////
// Join interest group in iframe tests.
////////////////////////////////////////////////////////////////////////////////

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let iframe = await createIframe(test, document.location.origin);

  // Join a same-origin InterestGroup in a iframe navigated to its origin.
  await runInIframe(test, iframe, `await joinInterestGroup(test_instance, "${uuid}");`);

  // Run an auction using window.location.origin as a bidder. The IG should
  // make a bid and win an auction.
  await runBasicFledgeTestExpectingWinner(test, uuid);
}, 'Join interest group in same-origin iframe, default permissions.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let iframe = await createIframe(test, OTHER_ORIGIN1);

  // Join a cross-origin InterestGroup in a iframe navigated to its origin.
  await runInIframe(test, iframe, `await joinInterestGroup(test_instance, "${uuid}");`);

  // Run an auction in this frame using the other origin as a bidder. The IG should
  // make a bid and win an auction.
  //
  // TODO: Once the permission defaults to not being able to join InterestGroups in
  // cross-origin iframes, this auction should have no winner.
  await runBasicFledgeTestExpectingWinner(
      test, uuid,
      { interestGroupBuyers: [OTHER_ORIGIN1],
        scoreAd: `if (browserSignals.interestGroupOwner !== "${OTHER_ORIGIN1}")
                    throw "Wrong owner: " + browserSignals.interestGroupOwner`
      });
}, 'Join interest group in cross-origin iframe, default permissions.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let iframe = await createIframe(test, OTHER_ORIGIN1, 'join-ad-interest-group');

  // Join a cross-origin InterestGroup in a iframe navigated to its origin.
  await runInIframe(test, iframe, `await joinInterestGroup(test_instance, "${uuid}");`);

  // Run an auction in this frame using the other origin as a bidder. The IG should
  // make a bid and win an auction.
  await runBasicFledgeTestExpectingWinner(
      test, uuid,
      { interestGroupBuyers: [OTHER_ORIGIN1],
        scoreAd: `if (browserSignals.interestGroupOwner !== "${OTHER_ORIGIN1}")
                    throw "Wrong owner: " + browserSignals.interestGroupOwner`
      });
}, 'Join interest group in cross-origin iframe with join-ad-interest-group permission.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let iframe = await createIframe(test, OTHER_ORIGIN1, "join-ad-interest-group 'none'");

  // Try to join an InterestGroup in a cross-origin iframe whose permissions policy
  // blocks joining interest groups. An exception should be thrown, and the interest
  // group should not be joined.
  await runInIframe(test, iframe,
                    `try {
                       await joinInterestGroup(test_instance, "${uuid}");
                     } catch (e) {
                       assert_true(e instanceof DOMException, "DOMException thrown");
                       assert_equals(e.name, "NotAllowedError", "NotAllowedError DOMException thrown");
                       return {result: "success"};
                     }
                     return "exception unexpectedly not thrown";`);

  // Run an auction in this frame using the other origin as a bidder. Since the join
  // should have failed, the auction should have no winner.
  await runBasicFledgeTestExpectingNoWinner(
      test, uuid,
      { interestGroupBuyers: [OTHER_ORIGIN1] });
}, 'Join interest group in cross-origin iframe with join-ad-interest-group permission denied.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let iframe = await createIframe(test, OTHER_ORIGIN1, 'join-ad-interest-group');

  // Try to join an IG with the parent's origin as an owner in a cross-origin iframe.
  // This should require a .well-known fetch to the parents origin, which will not
  // grant permission. The case where permission is granted is not yet testable.
  let interestGroup = JSON.stringify(createInterestGroupForOrigin(uuid, window.location.origin));
  await runInIframe(test, iframe,
                    `joinInterestGroup(test_instance, "${uuid}", ${interestGroup})`);

  // Run an auction with this page's origin as a bidder. Since the join
  // should have failed, the auction should have no winner.
  await runBasicFledgeTestExpectingNoWinner(test, uuid);
}, "Join interest group owned by parent's origin in cross-origin iframe.");

////////////////////////////////////////////////////////////////////////////////
// Run auction in iframe tests.
////////////////////////////////////////////////////////////////////////////////

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid);

  let iframe = await createIframe(test, document.location.origin);

  // Join a same-origin InterestGroup in a iframe navigated to its origin.
  await runInIframe(test, iframe, `await joinInterestGroup(test_instance, "${uuid}");`);

  // Run auction in same-origin iframe. This should succeed, by default.
  await runInIframe(
    test, iframe,
    `await runBasicFledgeTestExpectingWinner(test_instance, "${uuid}");`);
}, 'Run auction in same-origin iframe, default permissions.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  // Join an interest group owned by the the main frame's origin.
  await joinInterestGroup(test, uuid);

  let iframe = await createIframe(test, OTHER_ORIGIN1);

  // Run auction in cross-origin iframe. Currently, this is allowed by default.
  await runInIframe(
      test, iframe,
      `await runBasicFledgeTestExpectingWinner(
           test_instance, "${uuid}",
           {interestGroupBuyers: ["${window.location.origin}"]});`);
}, 'Run auction in cross-origin iframe, default permissions.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  // Join an interest group owned by the the main frame's origin.
  await joinInterestGroup(test, uuid);

  let iframe = await createIframe(test, OTHER_ORIGIN1, "run-ad-auction");

  // Run auction in cross-origin iframe that should allow the auction to occur.
  await runInIframe(
      test, iframe,
      `await runBasicFledgeTestExpectingWinner(
           test_instance, "${uuid}",
           {interestGroupBuyers: ["${window.location.origin}"]});`);
}, 'Run auction in cross-origin iframe with run-ad-auction permission.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  // No need to join any interest groups in this case - running an auction
  // should only throw an exception based on permissions policy, regardless
  // of whether there are any interest groups can participate.

  let iframe = await createIframe(test, OTHER_ORIGIN1, "run-ad-auction 'none'");

  // Run auction in cross-origin iframe that should not allow the auction to occur.
  await runInIframe(
      test, iframe,
      `try {
         await runBasicFledgeAuction(test_instance, "${uuid}");
       } catch (e) {
         assert_true(e instanceof DOMException, "DOMException thrown");
         assert_equals(e.name, "NotAllowedError", "NotAllowedError DOMException thrown");
         return {result: "success"};
       }
       throw "Attempting to run auction unexpectedly did not throw"`);
}, 'Run auction in cross-origin iframe with run-ad-auction permission denied.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  // Join an interest group owned by the the main frame's origin.
  await joinInterestGroup(test, uuid);

  let iframe = await createIframe(test, OTHER_ORIGIN1, `run-ad-auction ${OTHER_ORIGIN1}`);

  await runInIframe(
      test, iframe,
      `await runBasicFledgeTestExpectingWinner(
        test_instance, "${uuid}",
        { interestGroupBuyers: ["${window.location.origin}"],
          seller: "${OTHER_ORIGIN2}",
          decisionLogicURL: createDecisionScriptURL("${uuid}", {origin: "${OTHER_ORIGIN2}"})
        });`);
}, 'Run auction in cross-origin iframe with run-ad-auction for iframe origin, which is different from seller origin.');

////////////////////////////////////////////////////////////////////////////////
// Navigate fenced frame iframe tests.
////////////////////////////////////////////////////////////////////////////////

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Join an interest group and run an auction with a winner.
  await joinInterestGroup(test, uuid);
  let config = await runBasicFledgeTestExpectingWinner(test, uuid);

  // Try to navigate a fenced frame to the winning ad in a cross-origin iframe
  // with no fledge-related permissions.
  let iframe = await createIframe(
      test, OTHER_ORIGIN1, "join-ad-interest-group 'none'; run-ad-auction 'none'");
  await runInIframe(
      test, iframe,
      `await createAndNavigateFencedFrame(test_instance, param);`,
      /*param=*/config);
  await waitForObservedRequests(
      uuid, [createBidderReportURL(uuid), createSellerReportURL(uuid)]);
}, 'Run auction main frame, open winning ad in cross-origin iframe.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  let iframe = await createIframe(
      test, OTHER_ORIGIN1, "join-ad-interest-group; run-ad-auction");
  await runInIframe(
      test, iframe,
      `await joinInterestGroup(test_instance, "${uuid}");
       await runBasicFledgeAuctionAndNavigate(test_instance, "${uuid}");
       await waitForObservedRequests(
         "${uuid}", [createBidderReportURL("${uuid}"), createSellerReportURL("${uuid}")])`);
}, 'Run auction in cross-origin iframe and open winning ad in nested fenced frame.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Run an auction in an cross-origin iframe, and get the resulting FencedFrameConfig.
  let iframe = await createIframe(
      test, OTHER_ORIGIN1, "join-ad-interest-group; run-ad-auction");
  let config = await runInIframe(
      test, iframe,
      `await joinInterestGroup(test_instance, "${uuid}");
       let config = await runBasicFledgeTestExpectingWinner(test_instance, "${uuid}");
       return {result: "success", returnValue: config};`);
  assert_true(config != null, "Value not returned from auction in iframe");
  assert_true(config instanceof FencedFrameConfig,
    `Wrong value type returned from auction: ${config.constructor.type}`);

  // Loading the winning ad in a fenced frame that's a child of the main frame should
  // succeed.
  await createAndNavigateFencedFrame(test, config);
  await waitForObservedRequests(
      uuid,
      [ createBidderReportURL(uuid, '1', OTHER_ORIGIN1),
        createSellerReportURL(uuid, '1', OTHER_ORIGIN1)]);
}, 'Run auction in cross-origin iframe and open winning ad in a fenced frame child of the main frame.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Run an auction in an cross-origin iframe, and get the resulting FencedFrameConfig.
  let iframe = await createIframe(
      test, OTHER_ORIGIN1, "join-ad-interest-group; run-ad-auction");
  let config = await runInIframe(
      test, iframe,
      `await joinInterestGroup(test_instance, "${uuid}");
       let config = await runBasicFledgeTestExpectingWinner(test_instance, "${uuid}");
       return {result: "success", returnValue: config};`);
  assert_true(config != null, "Value not returned from auction in iframe");
  assert_true(config instanceof FencedFrameConfig,
    `Wrong value type returned from auction: ${config.constructor.type}`);

  // Try to navigate a fenced frame to the winning ad in a cross-origin iframe
  // with no fledge-related permissions. The iframe is a different origin from the
  // first cross-origin iframe.
  let iframe2 = await createIframe(
    test, OTHER_ORIGIN2, "join-ad-interest-group 'none'; run-ad-auction 'none'");
  await runInIframe(
      test, iframe2,
      `await createAndNavigateFencedFrame(test_instance, param);`,
      /*param=*/config);
  await waitForObservedRequests(
      uuid,
      [ createBidderReportURL(uuid, '1', OTHER_ORIGIN1),
        createSellerReportURL(uuid, '1', OTHER_ORIGIN1)]);
}, 'Run auction in cross-origin iframe and open winning ad in a fenced frame child of another cross-origin iframe.');
