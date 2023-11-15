// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=/common/subset-tests.js
// META: script=resources/fledge-util.sub.js
// META: timeout=long
// META: variant=?1-5
// META: variant=?6-10
// META: variant=?11-15
// META: variant=?16-last

"use strict";

// Creates a tracker URL for a component ad. These are fetched from component ad URLs.
function createComponentAdTrackerURL(uuid, id) {
  return createTrackerURL(window.location.origin, uuid, 'track_get',
                          `component_ad_${id}`)
}

// Returns a component ad render URL that fetches the corresponding component ad
// tracker URL.
function createComponentAdRenderURL(uuid, id) {
  return createRenderURL(
      uuid,
      `fetch("${createComponentAdTrackerURL(uuid, id)}");`);
}

// Runs a generic component ad loading test. It joins an interest group with a
// "numComponentAdsInInterestGroup" component ads. The IG will make a bid that
// potentially includes some of them. Then an auction will be run, component
// ads potentially will be loaded in nested fenced frame within the main frame,
// and the test will make sure that each component ad render URL that should have
// been loaded in an iframe was indeed loaded.
//
// Joins an interest group that has "numComponentAdsInInterestGroup" component ads.
//
// "componentAdsInBid" is a list of 0-based indices of which of those ads will be
// included in the bid. It may contain duplicate component ads. If it's null then the
// bid will have no adComponents field, while if it is empty, the bid will have an empty
// adComponents field.
//
// "componentAdsToLoad" is another list of 0-based ad components, but it's the index of
// fenced frame configs in the top frame ad's getNestedConfigs(). It may also contain
// duplicates to load a particular ad twice.
//
// If "adMetadata" is true, metadata is added to each component ad. Only integer metadata
// is used, relying on renderURL tests to cover other types of renderURL metadata.
async function runComponentAdLoadingTest(test, uuid, numComponentAdsInInterestGroup,
                                         componentAdsInBid, componentAdsToLoad,
                                         adMetadata = false) {
  let interestGroupAdComponents = [];
  for (let i = 0; i < numComponentAdsInInterestGroup; ++i) {
    const componentRenderURL = createComponentAdRenderURL(uuid, i);
    let adComponent = {renderURL: componentRenderURL};
    if (adMetadata)
      adComponent.metadata = i;
    interestGroupAdComponents.push(adComponent);
  }

  const renderURL = createRenderURL(
      uuid,
      `// "status" is passed to the beacon URL, to be verified by waitForObservedRequests().
       let status = "ok";
       const componentAds = window.fence.getNestedConfigs()
       if (componentAds.length != 20)
         status = "unexpected getNestedConfigs() length";
       for (let i of ${JSON.stringify(componentAdsToLoad)}) {
         let fencedFrame = document.createElement("fencedframe");
         fencedFrame.mode = "opaque-ads";
         fencedFrame.config = componentAds[i];
         document.body.appendChild(fencedFrame);
       }

       window.fence.reportEvent({eventType: "beacon",
                                 eventData: status,
                                 destination: ["buyer"]});`
      );

  let bid = {bid:1, render: renderURL};
  if (componentAdsInBid) {
    bid.adComponents = [];
    for (let index of componentAdsInBid) {
      bid.adComponents.push(interestGroupAdComponents[index].renderURL);
    }
  }

  // In these tests, the bidder should always request a beacon URL.
  let expectedTrackerURLs = [`${createBidderBeaconURL(uuid)}, body: ok`];
  // Figure out which, if any, elements of "componentAdsToLoad" correspond to
  // component ads listed in bid.adComponents, and for those ads, add a tracker URL
  // to "expectedTrackerURLs".
  if (componentAdsToLoad && bid.adComponents) {
    for (let index of componentAdsToLoad) {
      if (index < componentAdsInBid.length)
        expectedTrackerURLs.push(createComponentAdTrackerURL(uuid, componentAdsInBid[index]));
    }
  }

  await joinInterestGroup(
      test, uuid,
      { biddingLogicURL:
          createBiddingScriptURL({
              generateBid:
                  `let expectedAdComponents = ${JSON.stringify(interestGroupAdComponents)};
                   let adComponents = interestGroup.adComponents;
                   if (adComponents.length !== expectedAdComponents.length)
                     throw "Unexpected adComponents";
                   for (let i = 0; i < adComponents.length; ++i) {
                    if (adComponents[i].renderURL !== expectedAdComponents[i].renderURL ||
                        adComponents[i].metadata !== expectedAdComponents[i].metadata) {
                      throw "Unexpected adComponents";
                    }
                   }
                   return ${JSON.stringify(bid)}`,
              reportWin:
                  `registerAdBeacon({beacon: '${createBidderBeaconURL(uuid)}'});` }),
        ads: [{renderURL: renderURL}],
        adComponents: interestGroupAdComponents});

  if (!bid.adComponents || bid.adComponents.length === 0) {
    await runBasicFledgeAuctionAndNavigate(
      test, uuid,
      {decisionLogicURL: createDecisionScriptURL(
        uuid,
        { scoreAd: `if (browserSignals.adComponents !== undefined)
                      throw "adComponents should be undefined"`})});
  } else {
    await runBasicFledgeAuctionAndNavigate(
      test, uuid,
      {decisionLogicURL: createDecisionScriptURL(
        uuid,
        { scoreAd:
              `if (JSON.stringify(browserSignals.adComponents) !==
                       '${JSON.stringify(bid.adComponents)}') {
                 throw "Unexpected adComponents: " + JSON.stringify(browserSignals.adComponents);
               }`})});
  }

  await waitForObservedRequests(uuid, expectedTrackerURLs);
}

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  const renderURL = createRenderURL(
    uuid,
    `let status = "ok";
     const nestedConfigsLength = window.fence.getNestedConfigs().length
     // "getNestedConfigs()" should return a list of 20 configs, to avoid leaking
     // whether there were any component URLs to the page.
     if (nestedConfigsLength != 20)
       status = "unexpected getNestedConfigs() length: " + nestedConfigsLength;
     window.fence.reportEvent({eventType: "beacon",
                               eventData: status,
                               destination: ["buyer"]});`);
  await joinInterestGroup(
      test, uuid,
      { biddingLogicURL:
          createBiddingScriptURL({
              generateBid:
                  'if (interestGroup.componentAds !== undefined) throw "unexpected componentAds"',
              reportWin:
                  `registerAdBeacon({beacon: "${createBidderBeaconURL(uuid)}"});` }),
        ads: [{renderUrl: renderURL}]});
  await runBasicFledgeAuctionAndNavigate(
      test, uuid,
      {decisionLogicURL: createDecisionScriptURL(
        uuid,
        { scoreAd: `if (browserSignals.adComponents !== undefined)
                      throw "adComponents should be undefined"`})});
  await waitForObservedRequests(uuid, [`${createBidderBeaconURL(uuid)}, body: ok`]);
}, 'Group has no component ads, no adComponents in bid.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
      test,
      {uuid: uuid,
       interestGroupOverrides: {
           biddingLogicURL:
           createBiddingScriptURL({
               generateBid:
                   `return {bid: 1,
                            render: interestGroup.ads[0].renderUrl,
                            adComponents: []};`})}});
}, 'Group has no component ads, adComponents in bid is empty array.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runComponentAdLoadingTest(
      test, uuid, /*numComponentAdsInInterestGroup=*/2, /*componentAdsInBid=*/null,
      // Try to load ad components, even though there are none. This should load
      // about:blank in those frames, though that's not testible.
      // The waitForObservedRequests() call may see extra requests, racily, if
      // component ads not found in the bid are used.
      /*componentAdsToLoad=*/[0, 1]);
}, 'Group has component ads, but not used in bid (no adComponents field).');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runComponentAdLoadingTest(
      test, uuid, /*numComponentAdsInInterestGroup=*/2, /*componentAdsInBid=*/[],
      // Try to load ad components, even though there are none. This should load
      // about:blank in those frames, though that's not testible.
      // The waitForObservedRequests() call may see extra requests, racily, if
      // component ads not found in the bid are used.
      /*componentAdsToLoad=*/[0, 1]);
}, 'Group has component ads, but not used in bid (adComponents field empty array).');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runComponentAdLoadingTest(
      test, uuid, /*numComponentAdsInInterestGroup=*/2, /*componentAdsInBid=*/null,
      // Try to load ad components, even though there are none. This should load
      // about:blank in those frames, though that's not testible.
      // The waitForObservedRequests() call may see extra requests, racily, if
      // component ads not found in the bid are used.
      /*componentAdsToLoad=*/[0, 1], /*adMetadata=*/true);
}, 'Unused component ads with metadata.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: {
            biddingLogicURL:
                createBiddingScriptURL({
                    generateBid:
                        `return {bid: 1,
                                 render: interestGroup.ads[0].renderUrl,
                                 adComponents: ["https://random.url.test/"]};`}),
            adComponents: [{renderURL: createComponentAdRenderURL(uuid, 0)}]}});
}, 'Unknown component ad URL in bid.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: {
            biddingLogicURL:
                createBiddingScriptURL({
                    generateBid:
                        `return {bid: 1,
                                 render: interestGroup.ads[0].renderUrl,
                                 adComponents: [interestGroup.ads[0].renderUrl]};`}),
            adComponents: [{renderURL: createComponentAdRenderURL(uuid, 0)}]}});
}, 'Render URL used as component ad URL in bid.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: {
            biddingLogicURL:
                createBiddingScriptURL({
                    generateBid:
                        `return {bid: 1, render: interestGroup.adComponents[0].renderURL};`}),
            adComponents: [{renderURL: createComponentAdRenderURL(uuid, 0)}]}});
}, 'Component ad URL used as render URL.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runComponentAdLoadingTest(test, uuid, /*numComponentAdsInInterestGroup=*/2,
                                  /*componentAdsInBid=*/[0, 1], /*componentAdsToLoad=*/[0, 1]);
}, '2 of 2 component ads in bid and then shown.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runComponentAdLoadingTest(test, uuid, /*numComponentAdsInInterestGroup=*/2,
                                  /*componentAdsInBid=*/[0, 1], /*componentAdsToLoad=*/[0, 1],
                                  /*adMetadata=*/true);
}, '2 of 2 component ads in bid and then shown, with metadata.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runComponentAdLoadingTest(test, uuid, /*numComponentAdsInInterestGroup=*/20,
                                  /*componentAdsInBid=*/[3, 10], /*componentAdsToLoad=*/[0, 1]);
}, '2 of 20 component ads in bid and then shown.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const intsUpTo19 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19];
  await runComponentAdLoadingTest(test, uuid, /*numComponentAdsInInterestGroup=*/20,
                                  /*componentAdsInBid=*/intsUpTo19,
                                  /*componentAdsToLoad=*/intsUpTo19);
}, '20 of 20 component ads in bid and then shown.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runComponentAdLoadingTest(test, uuid, /*numComponentAdsInInterestGroup=*/20,
                                  /*componentAdsInBid=*/[1, 2, 3, 4, 5, 6],
                                  /*componentAdsToLoad=*/[1, 3]);
}, '6 of 20 component ads in bid, 2 shown.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  // It should be possible to load ads multiple times. Each loaded ad should request a new tracking
  // URLs, as they're fetched via XHRs, rather than reporting.
  await runComponentAdLoadingTest(test, uuid, /*numComponentAdsInInterestGroup=*/4,
                                  /*componentAdsInBid=*/[0, 1, 2, 3],
                                  /*componentAdsToLoad=*/[0, 1, 1, 0, 3, 3, 2, 2, 1, 0]);
}, '4 of 4 component ads shown multiple times.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runComponentAdLoadingTest(test, uuid, /*numComponentAdsInInterestGroup=*/2,
                                  /*componentAdsInBid=*/[0, 0, 0, 0],
                                  /*componentAdsToLoad=*/[0, 1, 2, 3]);
}, 'Same component ad used multiple times in bid.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  // The bid only has one component ad, but the renderURL tries to load 5 component ads.
  // The others should all be about:blank. Can't test that, so just make sure there aren't
  // more requests than expected, and there's no crash.
  await runComponentAdLoadingTest(test, uuid, /*numComponentAdsInInterestGroup=*/2,
                                  /*componentAdsInBid=*/[0],
                                  /*componentAdsToLoad=*/[4, 3, 2, 1, 0]);
}, 'Load component ads not in bid.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid);

  let adComponents = [];
  let adComponentsList = [];
  for (let i = 0; i < 21; ++i) {
    let componentRenderURL = createComponentAdTrackerURL(uuid, i);
    adComponents.push({renderURL: componentRenderURL});
    adComponentsList.push(componentRenderURL);
  }

  await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: {
            biddingLogicURL:
                createBiddingScriptURL({
                    generateBid:
                        `return {bid: 1,
                                 render: "${renderURL}",
                                 adComponents: ${JSON.stringify(adComponentsList)}};`}),
            ads: [{renderURL: renderURL}],
            adComponents: adComponents}});
}, '21 component ads not allowed in bid.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const renderURL = createRenderURL(uuid);

  let adComponents = [];
  let adComponentsList = [];
  for (let i = 0; i < 21; ++i) {
    let componentRenderURL = createComponentAdTrackerURL(uuid, i);
    adComponents.push({renderURL: componentRenderURL});
    adComponentsList.push(adComponents[0].renderURL);
  }

  await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: {
            biddingLogicURL:
                createBiddingScriptURL({
            generateBid:
                `return {bid: 1,
                         render: "${renderURL}",
                         adComponents: ${JSON.stringify(adComponentsList)}};`}),
            ads: [{renderURL: renderURL}],
            adComponents: adComponents}});
}, 'Same component ad not allowed 21 times in bid.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // The component ad's render URL will try to send buyer and seller reports,
  // which should not be sent (but not throw an exception), and then request a
  // a tracker URL via fetch, which should be requested from the server.
  const componentRenderURL =
      createRenderURL(
        uuid,
        `window.fence.reportEvent({eventType: "beacon",
                                   eventData: "Should not be sent",
                                   destination: ["buyer", "seller"]});
         fetch("${createComponentAdTrackerURL(uuid, 0)}");`);

  const renderURL = createRenderURL(
      uuid,
      `let fencedFrame = document.createElement("fencedframe");
       fencedFrame.mode = "opaque-ads";
       fencedFrame.config = window.fence.getNestedConfigs()[0];
       document.body.appendChild(fencedFrame);

       async function waitForRequestAndSendBeacons() {
         // Wait for the nested fenced frame to request its tracker URL.
         await waitForObservedRequests("${uuid}", ["${createComponentAdTrackerURL(uuid, 0)}"]);

         // Now that the tracker URL has been received, the component ad has tried to
         // send a beacon, so have the main renderURL send a beacon, which should succeed
         // and should hopefully be sent after the component ad's beacon, if it was
         // going to (incorrectly) send one.
         window.fence.reportEvent({eventType: "beacon",
                                   eventData: "top-ad",
                                   destination: ["buyer", "seller"]});
       }
       waitForRequestAndSendBeacons();`);

  await joinInterestGroup(
      test, uuid,
      { biddingLogicURL:
          createBiddingScriptURL({
              generateBid:
                  `return {bid: 1,
                           render: "${renderURL}",
                           adComponents: ["${componentRenderURL}"]};`,
              reportWin:
                  `registerAdBeacon({beacon: '${createBidderBeaconURL(uuid)}'});` }),
        ads: [{renderURL: renderURL}],
        adComponents: [{renderURL: componentRenderURL}]});

  await runBasicFledgeAuctionAndNavigate(
    test, uuid,
    {decisionLogicURL: createDecisionScriptURL(
        uuid,
        { reportResult: `registerAdBeacon({beacon: '${createSellerBeaconURL(uuid)}'});` }) });

  // Only the renderURL should have sent any beacons, though the component ad should have sent
  // a tracker URL fetch request.
  await waitForObservedRequests(uuid, [createComponentAdTrackerURL(uuid, 0),
                                       `${createBidderBeaconURL(uuid)}, body: top-ad`,
                                       `${createSellerBeaconURL(uuid)}, body: top-ad`]);


}, 'Reports not sent from component ad.');
