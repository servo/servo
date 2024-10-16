// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-4
// META: variant=?5-8
// META: variant=?9-last

"use strict;"

// These are separate from the other join-leave tests because these all create
// and navigate fenced frames, which is much slower than just joining/leaving
// interest groups, and running the occasional auction. Most tests use a
// buyer with an origin of OTHER_ORIGIN1, so it has a distinct origin from the
// seller and publisher.

// Creates a tracker URL that's requested when a call succeeds in a fenced
// frame.
function createSuccessURL(uuid, origin = document.location.origin) {
  return createTrackerURL(origin, uuid, "track_get", "success");
}

// Creates a tracker URL that's requested when a call fails in a fenced frame, with
// the expected exception.
function createExceptionURL(uuid, origin = document.location.origin) {
  return createTrackerURL(origin, uuid, "track_get", "exception");
}

// Creates a tracker URL that's requested when joinAdInterestGroup() or
// leaveAdInterestGroup() fails with an exception other than the one that's
// expected.
function createBadExceptionURL(uuid, origin = document.location.origin) {
  return createTrackerURL(origin, uuid, "track_get", "bad_exception");
}

// Creates render URL that calls "navigator.leaveAdInterestGroup()" when
// loaded, with no arguments. It then fetches a URL depending on whether it
// threw an exception. No exception should ever be thrown when this is run
// in an ad URL, so only fetch the "bad exception" URL on error.
function createNoArgsTryLeaveRenderURL(uuid, origin = document.location.origin) {
  return createRenderURL(
    uuid,
    `async function TryLeave() {
       try {
         await navigator.leaveAdInterestGroup();
         await fetch("${createSuccessURL(uuid, origin)}");
       } catch (e) {
         await fetch("${createBadExceptionURL(uuid, origin)}");
       }
     }

     TryLeave();`,
    /*signalsParams=*/null,
    origin);
}

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Interest group that an ad fenced frame attempts to join. The join should
  // fail.
  let interestGroupJoinedInFrame = createInterestGroupForOrigin(
      uuid, document.location.origin, {name: 'group2'});

  // Create a render URL that tries to join "interestGroupJoinedInFrame".
  const renderURL = createRenderURL(
    uuid,
    `async function TryJoin() {
       try {
         await navigator.joinAdInterestGroup(
             ${JSON.stringify(interestGroupJoinedInFrame)});
         await fetch("${createSuccessURL(uuid)}");
       } catch (e) {
         if (e instanceof DOMException && e.name === "NotAllowedError") {
           await fetch("${createExceptionURL(uuid)}");
         } else {
           await fetch("${createBadExceptionURL(uuid)}");
         }
       }
     }

     TryJoin();`);

  await joinInterestGroup(test, uuid, {ads: [{ renderURL: renderURL}]});

  await runBasicFledgeAuctionAndNavigate(test, uuid);

  // This should wait until the leave call has thrown an exception.
  await waitForObservedRequests(
      uuid,
      [createBidderReportURL(uuid), createSellerReportURL(uuid), createExceptionURL(uuid)]);

  // Leave the initial interest group.
  await leaveInterestGroup();

  // Check the interest group was not successfully joined in the fenced frame
  // by running an auction, to make sure the thrown exception accurately
  // indicates the group wasn't joined.
  await runBasicFledgeTestExpectingNoWinner(test, uuid);
}, 'joinAdInterestGroup() in ad fenced frame.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Create a render URL that tries to leave the default test interest group by
  // name. Even a though a render URL can leave its own interest group by using
  // the 0-argument version of leaveAdInterestGroup(), it can't leave its own
  // interest group by using the 1-argument version, so this should fail.
  const renderURL = createRenderURL(
      uuid,
      `async function TryLeave() {
         try {
           await navigator.leaveAdInterestGroup(
               {owner: "${window.location.origin}", name: "${DEFAULT_INTEREST_GROUP_NAME}"});
           await fetch("${createSuccessURL(uuid)}");
         } catch (e) {
           if (e instanceof DOMException && e.name === "NotAllowedError") {
             await fetch("${createExceptionURL(uuid)}");
           } else {
             await fetch("${createBadExceptionURL(uuid)}");
           }
         }
       }

       TryLeave();`);

  await joinInterestGroup(
      test, uuid,
      {ads: [{ renderURL: renderURL}]});

  await runBasicFledgeAuctionAndNavigate(test, uuid);

  // This should wait until the leave call has thrown an exception.
  await waitForObservedRequests(
      uuid,
      [createBidderReportURL(uuid), createSellerReportURL(uuid), createExceptionURL(uuid)]);

  // Check the interest group was not left.
  await runBasicFledgeTestExpectingWinner(test, uuid);
}, 'leaveAdInterestGroup() in ad fenced frame, specify an interest group.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  const bidder_origin = OTHER_ORIGIN1;
  const render_url_origin = window.location.origin;

  await joinCrossOriginInterestGroup(
      test, uuid, bidder_origin,
      {ads: [{ renderURL: createNoArgsTryLeaveRenderURL(uuid, render_url_origin) }]});

  await runBasicFledgeAuctionAndNavigate(test, uuid, {interestGroupBuyers : [bidder_origin]});

  // Leaving the interest group should claim to succeed, to avoid leaking
  // whether or not the buyer was same-origin or to the fenced frame.
  await waitForObservedRequests(
      uuid,
      [ createBidderReportURL(uuid), createSellerReportURL(uuid),
        createSuccessURL(uuid, render_url_origin)]);

  // Check the interest group was not actually left.
  await runBasicFledgeTestExpectingWinner(test, uuid, {interestGroupBuyers : [bidder_origin]});
}, 'leaveAdInterestGroup() in non-buyer origin ad fenced frame, no parameters.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  const bidder_origin = OTHER_ORIGIN1;
  const render_url_origin = OTHER_ORIGIN1;

  // Use a different origin for the buyer, to make sure that's the origin
  // that matters.
  await joinCrossOriginInterestGroup(
      test, uuid, bidder_origin,
      {ads: [{ renderURL: createNoArgsTryLeaveRenderURL(uuid, render_url_origin) }]});

  await runBasicFledgeAuctionAndNavigate(test, uuid, {interestGroupBuyers : [bidder_origin]});

  // This should wait until the leave call has completed.
  await waitForObservedRequests(
      uuid,
      [ createBidderReportURL(uuid), createSellerReportURL(uuid),
        createSuccessURL(uuid, render_url_origin)]);

  // Check the interest group was actually left.
  await runBasicFledgeTestExpectingNoWinner(test, uuid, {interestGroupBuyers : [bidder_origin]});
}, 'leaveAdInterestGroup() in buyer origin ad fenced frame, no parameters.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  const bidder_origin = OTHER_ORIGIN1;
  const render_url_origin = OTHER_ORIGIN1;
  const iframe_origin = OTHER_ORIGIN1;

  // Create a render URL which, in an iframe, loads the common "try leave"
  // render URL from the buyer's origin (which isn't technically being used as
  // a render URL, in this case).
  const renderURL = createRenderURL(
      uuid,
      `let iframe = document.createElement("iframe");
       iframe.permissions = "join-ad-interest-group";
       iframe.src = "${createNoArgsTryLeaveRenderURL(uuid, iframe_origin)}";
       document.body.appendChild(iframe);`,
      /*signalsParams=*/null,
      render_url_origin);

  await joinCrossOriginInterestGroup(
      test, uuid, bidder_origin,
      {ads: [{ renderURL: renderURL }]});

  await runBasicFledgeAuctionAndNavigate(test, uuid, {interestGroupBuyers : [bidder_origin]});

  // This should wait until the leave call has completed.
  await waitForObservedRequests(
      uuid,
      [ createBidderReportURL(uuid), createSellerReportURL(uuid),
        createSuccessURL(uuid, iframe_origin)]);

  // Check the interest group was actually left.
  await runBasicFledgeTestExpectingNoWinner(test, uuid, {interestGroupBuyers : [bidder_origin]});
}, 'leaveAdInterestGroup() in same-origin iframe inside buyer origin ad fenced frame, no parameters.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  const bidder_origin = OTHER_ORIGIN1;
  const render_url_origin = OTHER_ORIGIN1;
  const iframe_origin = document.location.origin;

  // Create a render URL which, in an iframe, loads the common "try leave"
  // render URL from an origin other than the buyer's origin.
  const renderURL = createRenderURL(
      uuid,
      `let iframe = document.createElement("iframe");
       iframe.permissions = "join-ad-interest-group";
       iframe.src = "${createNoArgsTryLeaveRenderURL(uuid, iframe_origin)}";
       document.body.appendChild(iframe);`,
      /*signalsParams=*/null,
      render_url_origin);

  await joinCrossOriginInterestGroup(
      test, uuid, bidder_origin,
      {ads: [{ renderURL: renderURL }]});

  await runBasicFledgeAuctionAndNavigate(test, uuid, {interestGroupBuyers : [bidder_origin]});

  // Leaving the interest group should claim to succeed, to avoid leaking
  // whether or not the buyer was same-origin or to the iframe.
  await waitForObservedRequests(
      uuid,
      [ createBidderReportURL(uuid), createSellerReportURL(uuid),
        createSuccessURL(uuid, iframe_origin)]);

  // Check the interest group was not actually left.
  await runBasicFledgeTestExpectingWinner(test, uuid, {interestGroupBuyers : [bidder_origin]});
}, 'leaveAdInterestGroup() in cross-origin iframe inside buyer origin ad fenced frame, no parameters.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  const bidder_origin = OTHER_ORIGIN1;
  const render_url_origin = document.location.origin;
  const iframe_origin = document.location.origin;

  // Create a render URL which, in an iframe, loads the common "try leave"
  // render URL from an origin other than the buyer's origin (which isn't
  // technically being used as a render URL, in this case).
  const renderURL = createRenderURL(
      uuid,
      `let iframe = document.createElement("iframe");
       iframe.permissions = "join-ad-interest-group";
       iframe.src = "${createNoArgsTryLeaveRenderURL(uuid, iframe_origin)}";
       document.body.appendChild(iframe);`,
      /*signalsParams=*/null,
      render_url_origin);

  await joinCrossOriginInterestGroup(
      test, uuid, bidder_origin,
      {ads: [{ renderURL: renderURL }]});

  await runBasicFledgeAuctionAndNavigate(test, uuid, {interestGroupBuyers : [bidder_origin]});

  // Leaving the interest group should claim to succeed, to avoid leaking
  // whether or not the buyer was same-origin or to the fenced frame.
  await waitForObservedRequests(
      uuid,
      [ createBidderReportURL(uuid), createSellerReportURL(uuid),
        createSuccessURL(uuid, iframe_origin)]);

  // Check the interest group was not actually left.
  await runBasicFledgeTestExpectingWinner(test, uuid, {interestGroupBuyers : [bidder_origin]});
}, 'leaveAdInterestGroup() in same-origin iframe inside non-buyer origin ad fenced frame, no parameters.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  const bidder_origin = OTHER_ORIGIN1;
  const render_url_origin = document.location.origin;
  const iframe_origin = OTHER_ORIGIN1;

  // Create a render URL which, in an iframe, loads the common "try leave"
  // render URL from the buyer's origin (which isn't technically being used as
  // a render URL, in this case).
  const renderURL = createRenderURL(
      uuid,
      `let iframe = document.createElement("iframe");
       iframe.permissions = "join-ad-interest-group";
       iframe.src = "${createNoArgsTryLeaveRenderURL(uuid, iframe_origin)}";
       document.body.appendChild(iframe);`,
      /*signalsParams=*/null,
      render_url_origin);

  await joinCrossOriginInterestGroup(
      test, uuid, bidder_origin,
      {ads: [{ renderURL: renderURL }]});

  await runBasicFledgeAuctionAndNavigate(test, uuid, {interestGroupBuyers : [bidder_origin]});
  // Leaving the interest group should succeed.
  await waitForObservedRequests(
      uuid,
      [ createBidderReportURL(uuid), createSellerReportURL(uuid),
        createSuccessURL(uuid, iframe_origin)]);

  // Check the interest group was left.
  await runBasicFledgeTestExpectingNoWinner(test, uuid, {interestGroupBuyers : [bidder_origin]});
}, 'leaveAdInterestGroup() in cross-origin buyer iframe inside non-buyer origin ad fenced frame, no parameters.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Render URL that loads the first ad component in a nested fenced frame.
  let loadFirstComponentAdURL =
      createRenderURL(
        uuid,
        `let fencedFrame = document.createElement("fencedframe");
         fencedFrame.mode = "opaque-ads";
         fencedFrame.config = window.fence.getNestedConfigs()[0];
         document.body.appendChild(fencedFrame);`,
        /*signalsParams=*/null,
        OTHER_ORIGIN1);

  await joinInterestGroup(
      test, uuid,
      // Interest group that makes a bid with a component ad. The render URL
      // will open the component ad in a fenced frame, and the component ad
      // URL is the common URL that tries to leave the ad's current interest
      // group, reporting the result to a tracker URL.
      { biddingLogicURL: createBiddingScriptURL(
          { generateBid: `return {
                            bid: 1,
                            render: interestGroup.ads[0].renderURL,
                            adComponents: [interestGroup.adComponents[0].renderURL]
                          };` }),
        ads: [{ renderURL: loadFirstComponentAdURL }],
        adComponents: [{ renderURL: createNoArgsTryLeaveRenderURL(uuid) }]});

  await runBasicFledgeAuctionAndNavigate(test, uuid);

  // Leaving the interest group should claim to succeed, to avoid leaking
  // whether or not the buyer was same-origin or to the fenced frame.
  await waitForObservedRequests(
      uuid,
      [createSellerReportURL(uuid), createSuccessURL(uuid)]);

  // Check the interest group was left.
  await runBasicFledgeTestExpectingNoWinner(test, uuid);
}, 'leaveAdInterestGroup() in component ad fenced frame, no parameters.');
