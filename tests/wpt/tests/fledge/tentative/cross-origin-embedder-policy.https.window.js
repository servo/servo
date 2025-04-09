// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: timeout=long

"use strict";

promise_test(async test => {
  const uuid = generateUuid(test);

  let otherWindow = await createTopLevelWindow(test, OTHER_ORIGIN1);

  // Join a cross-origin InterestGroup in a top-level window navigated to its origin.
  // Has to be top-level to avoid being subject to the Cross-Origin-Embedder-Policy
  // of this page.
  await runInFrame(test, otherWindow,
      `await joinInterestGroup(test_instance, "${uuid}");`);

  // Run an auction in this frame using the other origin as a bidder. The bidding
  // script load should not be blocked by the COEP that blocks cross-origin
  // resources.
  await runBasicFledgeTestExpectingWinner(
    test, uuid,
    { interestGroupBuyers: [OTHER_ORIGIN1] });
}, 'COEP does not block bidder scripts.');

promise_test(async test => {
  const uuid = generateUuid(test);

  // Run an auction with a cross-origin seller script, it should not be blocked
  // by COEP.
  await joinGroupAndRunBasicFledgeTestExpectingWinner(
    test,
    { uuid,
      auctionConfigOverrides : {seller: OTHER_ORIGIN1,
        decisionLogicURL: createDecisionScriptURL(uuid, { origin: OTHER_ORIGIN1 })
    }});
}, 'COEP does not block seller scripts.');
