// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=/common/subset-tests.js
// META: script=resources/fledge-util.sub.js
// META: timeout=long
// META: variant=?1-4
// META: variant=?5-8
// META: variant=?9-12
// META: variant=?13-last

"use strict;"

////////////////////////////////////////////////////////////////////////////////
// Join interest group in iframe tests.
////////////////////////////////////////////////////////////////////////////////

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let iframe = await createIframe(test, document.location.origin);

  // Join a same-origin InterestGroup in a iframe navigated to its origin.
  await runInFrame(test, iframe, `await joinInterestGroup(test_instance, "${uuid}");`);

  // Run an auction using window.location.origin as a bidder. The IG should
  // make a bid and win an auction.
  await runBasicFledgeTestExpectingWinner(test, uuid);
}, 'Join interest group in same-origin iframe, default permissions.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let iframe = await createIframe(test, OTHER_ORIGIN1);

  // Join a cross-origin InterestGroup in a iframe navigated to its origin.
  await runInFrame(test, iframe, `await joinInterestGroup(test_instance, "${uuid}");`);

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
  await runInFrame(test, iframe, `await joinInterestGroup(test_instance, "${uuid}");`);

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
  await runInFrame(test, iframe,
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
  await runInFrame(test, iframe,
                   `try {
                      await joinInterestGroup(test_instance, "${uuid}", ${interestGroup});
                    } catch (e) {
                      assert_true(e instanceof DOMException, "DOMException thrown");
                      assert_equals(e.name, "NotAllowedError", "NotAllowedError DOMException thrown");
                      return {result: "success"};
                    }
                    return "exception unexpectedly not thrown";`);

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
  await runInFrame(test, iframe, `await joinInterestGroup(test_instance, "${uuid}");`);

  // Run auction in same-origin iframe. This should succeed, by default.
  await runInFrame(
    test, iframe,
    `await runBasicFledgeTestExpectingWinner(test_instance, "${uuid}");`);
}, 'Run auction in same-origin iframe, default permissions.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  // Join an interest group owned by the the main frame's origin.
  await joinInterestGroup(test, uuid);

  let iframe = await createIframe(test, OTHER_ORIGIN1);

  // Run auction in cross-origin iframe. Currently, this is allowed by default.
  await runInFrame(
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
  await runInFrame(
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
  await runInFrame(
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

  await runInFrame(
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
  await runInFrame(
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
  await runInFrame(
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
  let config = await runInFrame(
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
  let config = await runInFrame(
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
  await runInFrame(
      test, iframe2,
      `await createAndNavigateFencedFrame(test_instance, param);`,
      /*param=*/config);
  await waitForObservedRequests(
      uuid,
      [ createBidderReportURL(uuid, '1', OTHER_ORIGIN1),
        createSellerReportURL(uuid, '1', OTHER_ORIGIN1)]);
}, 'Run auction in cross-origin iframe and open winning ad in a fenced frame child of another cross-origin iframe.');

////////////////////////////////////////////////////////////////////////////////
// Other tests.
////////////////////////////////////////////////////////////////////////////////

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  let iframe = await createIframe(test, OTHER_ORIGIN1, "run-ad-auction");

  // Do everything in a cross-origin iframe, and make sure correct top-frame origin is used.
  await runInFrame(
      test, iframe,
      `const uuid = "${uuid}";
       const renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'hostname');

       await joinInterestGroup(
          test_instance, uuid,
          { trustedBiddingSignalsKeys: ['hostname'],
            trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL,
            ads: [{ renderURL: renderURL }],
            biddingLogicURL: createBiddingScriptURL({
              generateBid:
                  \`if (browserSignals.topWindowHostname !== "${document.location.hostname}")
                      throw "Wrong topWindowHostname: " + browserSignals.topWindowHostname;
                    if (trustedBiddingSignals.hostname !== '${window.location.hostname}')
                      throw 'Wrong hostname: ' + trustedBiddingSignals.hostname;\`})});

       await runBasicFledgeTestExpectingWinner(
           test_instance, uuid,
           { trustedScoringSignalsURL: TRUSTED_SCORING_SIGNALS_URL,
            decisionLogicURL:
            createDecisionScriptURL(
              uuid,
              { scoreAd:
                    \`if (browserSignals.topWindowHostname !== "${document.location.hostname}")
                        throw "Wrong topWindowHostname: " + browserSignals.topWindowHostname;
                      if (trustedScoringSignals.renderURL["\${renderURL}"] !== '${window.location.hostname}')
                        throw 'Wrong hostname: ' + trustedScoringSignals.renderURL["\${renderURL}"];\` })});`);
}, 'Different top-frame origin.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  let bidderOrigin = OTHER_ORIGIN1;
  let sellerOrigin = OTHER_ORIGIN2;
  let bidderSendReportToURL = createBidderReportURL(uuid, '1', OTHER_ORIGIN3);
  let sellerSendReportToURL = createSellerReportURL(uuid, '2', OTHER_ORIGIN4);
  let bidderBeaconURL = createBidderBeaconURL(uuid, '3', OTHER_ORIGIN5);
  let sellerBeaconURL = createSellerBeaconURL(uuid, '4', OTHER_ORIGIN6);
  let renderURL = createRenderURL(
      uuid,
      `window.fence.reportEvent({
         eventType: "beacon",
         eventData: window.location.href,
         destination: ["buyer", "seller"]
       })`,
       /*signalsParams=*/null, OTHER_ORIGIN7);

  let iframe = await createIframe(test, bidderOrigin, "join-ad-interest-group");
  let interestGroup = createInterestGroupForOrigin(
      uuid, bidderOrigin,
      {biddingLogicURL: createBiddingScriptURL(
        { origin: bidderOrigin,
          generateBid: `if (browserSignals.topWindowHostname !== "${document.location.hostname}")
                          throw "Wrong topWindowHostname: " + browserSignals.topWindowHostname;
                        if (interestGroup.owner !== "${bidderOrigin}")
                          throw "Wrong origin: " + interestGroup.owner;
                        if (!interestGroup.biddingLogicURL.startsWith("${bidderOrigin}"))
                          throw "Wrong origin: " + interestGroup.biddingLogicURL;
                        if (interestGroup.ads[0].renderURL !== "${renderURL}")
                          throw "Wrong renderURL: " + interestGroup.ads[0].renderURL;
                        if (browserSignals.seller !== "${sellerOrigin}")
                          throw "Wrong origin: " + browserSignals.seller;`,
          reportWin: `if (browserSignals.topWindowHostname !== "${document.location.hostname}")
                        throw "Wrong topWindowHostname: " + browserSignals.topWindowHostname;
                      if (browserSignals.seller !== "${sellerOrigin}")
                        throw "Wrong seller: " + browserSignals.seller;
                      if (browserSignals.interestGroupOwner !== "${bidderOrigin}")
                        throw "Wrong interestGroupOwner: " + browserSignals.interestGroupOwner;
                      if (browserSignals.renderURL !== "${renderURL}")
                        throw "Wrong renderURL: " + browserSignals.renderURL;
                      if (browserSignals.seller !== "${sellerOrigin}")
                        throw "Wrong seller: " + browserSignals.seller;
                      sendReportTo("${bidderSendReportToURL}");
                      registerAdBeacon({beacon: "${bidderBeaconURL}"});` }),
       ads: [{ renderURL: renderURL }]});
  await runInFrame(
      test, iframe,
      `await joinInterestGroup(test_instance, "${uuid}", ${JSON.stringify(interestGroup)});`);

  await runBasicFledgeAuctionAndNavigate(test, uuid,
    { seller: sellerOrigin,
      interestGroupBuyers: [bidderOrigin],
      decisionLogicURL: createDecisionScriptURL(
        uuid,
        { origin: sellerOrigin,
          scoreAd: `if (browserSignals.topWindowHostname !== "${document.location.hostname}")
                      throw "Wrong topWindowHostname: " + browserSignals.topWindowHostname;
                    if (auctionConfig.seller !== "${sellerOrigin}")
                      throw "Wrong seller: " + auctionConfig.seller;
                    if (auctionConfig.interestGroupBuyers[0] !== "${bidderOrigin}")
                      throw "Wrong interestGroupBuyers: " + auctionConfig.interestGroupBuyers;
                    if (browserSignals.interestGroupOwner !== "${bidderOrigin}")
                      throw "Wrong interestGroupOwner: " + browserSignals.interestGroupOwner;
                    if (browserSignals.renderURL !== "${renderURL}")
                      throw "Wrong renderURL: " + browserSignals.renderURL;`,
          reportResult: `if (browserSignals.topWindowHostname !== "${document.location.hostname}")
                           throw "Wrong topWindowHostname: " + browserSignals.topWindowHostname;
                         if (browserSignals.interestGroupOwner !== "${bidderOrigin}")
                           throw "Wrong interestGroupOwner: " + browserSignals.interestGroupOwner;
                         if (browserSignals.renderURL !== "${renderURL}")
                           throw "Wrong renderURL: " + browserSignals.renderURL;
                         sendReportTo("${sellerSendReportToURL}");
                         registerAdBeacon({beacon: "${sellerBeaconURL}"});`})
     });

  await waitForObservedRequests(
      uuid,
      [ bidderSendReportToURL,
        sellerSendReportToURL,
        `${bidderBeaconURL}, body: ${renderURL}`,
        `${sellerBeaconURL}, body: ${renderURL}`
      ]);
}, 'Single seller auction with as many distinct origins as possible (except no component ads).');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Join an interest group and run an auction with a winner. Use a tracking
  // URL for the ad, so that if it's incorrectly loaded in this test, the
  // waitForObservedRequests() at the end of the test will see it, and the
  // test will fail.
  await joinInterestGroup(
      test, uuid,
      {ads: [{renderURL: createTrackerURL(window.location.origin, uuid, 'track_get', 'renderURL')}]});
  let config = await runBasicFledgeTestExpectingWinner(test, uuid);

  // Try to navigate a fenced frame to the winning ad in a new same-origin
  // window. This should fail. Unfortunately, there's no assertion that
  // can be checked for, and can't communicate with the contents of the
  // fenced frame to make sure the load fails.
  //
  // So instead, join an interest group with a different sendReportTo-url,
  // overwriting the previously joined one, and run another auction, loading
  // the winner in another fenced frame.
  //
  // Then wait to see that only the reporting URLs from that second auction
  // are requested. They should almost always be requested after the URLs
  // from the first auction.
  let child_window =
      await createFrame(test, document.location.origin, /*is_iframe=*/false);
  await runInFrame(
      test, child_window,
      `await createAndNavigateFencedFrame(test_instance, param);
       await joinInterestGroup(
          test_instance, "${uuid}",
          {biddingLogicURL: createBiddingScriptURL(
              {reportWin: "sendReportTo('${createBidderReportURL(uuid, "2")}');" })});
       await runBasicFledgeAuctionAndNavigate(test_instance, "${uuid}");`,
      /*param=*/config);
  await waitForObservedRequests(
      uuid, [createBidderReportURL(uuid, "2"), createSellerReportURL(uuid)]);
}, 'Run auction in main frame, try to open winning ad in different same-origin main frame.');
