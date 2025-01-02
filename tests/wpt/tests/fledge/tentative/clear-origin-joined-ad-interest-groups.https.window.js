// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-4
// META: variant=?5-8
// META: variant=?9-12
// META: variant=?13-last

"use strict;"

///////////////////////////////////////////////////////////////////////////////
// Basic tests with no interest groups joined.
///////////////////////////////////////////////////////////////////////////////

subsetTest(promise_test, async test => {
  await navigator.clearOriginJoinedAdInterestGroups(window.location.origin);
}, 'clearOriginJoinedAdInterestGroups(), no groups joined, no group list.');

subsetTest(promise_test, async test => {
  await navigator.clearOriginJoinedAdInterestGroups(window.location.origin, []);
}, 'clearOriginJoinedAdInterestGroups(), no groups joined, group list.');

subsetTest(promise_test, async test => {
  try {
    await navigator.clearOriginJoinedAdInterestGroups(OTHER_ORIGIN1);
    throw 'Exception unexpectedly not thrown';
  } catch (e) {
    if (!(e instanceof DOMException) || e.name !== 'NotAllowedError') {
      throw 'Wrong exception thrown: ' + e.toString();
    }
  }
}, 'clearOriginJoinedAdInterestGroups(), cross-origin, no groups joined, no group list.');

subsetTest(promise_test, async test => {
  try {
    await navigator.clearOriginJoinedAdInterestGroups(OTHER_ORIGIN1, []);
    throw 'Exception unexpectedly not thrown';
  } catch (e) {
    if (!(e instanceof DOMException) || e.name !== 'NotAllowedError') {
      throw 'Wrong exception thrown: ' + e.toString();
    }
  }
}, 'clearOriginJoinedAdInterestGroups(), cross-origin, no groups joined, group list.');

///////////////////////////////////////////////////////////////////////////////
// Tests where interest groups are all owned by document.location.origin.
///////////////////////////////////////////////////////////////////////////////

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Join 3 groups.
  await joinInterestGroup(test, uuid);
  await joinInterestGroup(test, uuid, {name: 'group 2'});
  await joinInterestGroup(test, uuid, {name: 'group 3'});

  // A single clear should leave them all.
  await navigator.clearOriginJoinedAdInterestGroups(window.location.origin);

  // Confirm that they were left.
  await runBasicFledgeTestExpectingNoWinner(test, uuid);
}, 'clearOriginJoinedAdInterestGroups(), multiple groups joined, no group list.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  let group1ReportURL = createBidderReportURL(uuid, /*id=*/'1');
  let group2ReportURL = createBidderReportURL(uuid, /*id=*/'2');
  let group3ReportURL = createBidderReportURL(uuid, /*id=*/'3');

  // Join 3 groups, with distinct report URLs and increasing bid amounts.
  // Set "executionMode" to "group-by-origin" for two of them, since cross-origin
  // leaves removes all groups joined from the other origin with that execution
  // mode. Since clearOriginJoinedAdInterestGroups() only leaves interest
  // groups joined on the current origin, the executionMode should not matter.
  await joinInterestGroup(
      test, uuid,
      { name: 'group 1',
        executionMode: 'group-by-origin',
        biddingLogicURL: createBiddingScriptURL(
            { bid: 1, reportWin: `sendReportTo("${group1ReportURL}");`})});
  await joinInterestGroup(
      test, uuid,
      { name: 'group 2',
        biddingLogicURL: createBiddingScriptURL(
            { bid: 2, reportWin: `sendReportTo("${group2ReportURL}");`})});
  await joinInterestGroup(
      test, uuid,
      { name: 'group 3',
        executionMode: 'group-by-origin',
        biddingLogicURL: createBiddingScriptURL(
            { bid: 3, reportWin: `sendReportTo("${group3ReportURL}");`})});

  // Group 3 should win an auction, since it bids the most.
  await runBasicFledgeAuctionAndNavigate(test, uuid);
  await waitForObservedRequests(
      uuid, [group3ReportURL, createSellerReportURL(uuid)]);
  await fetch(createCleanupURL(uuid));

  // Clear, leaving group 1 in place, and run an auction, which group 1 should win.
  await navigator.clearOriginJoinedAdInterestGroups(
      window.location.origin, ['group 1']);
  await runBasicFledgeAuctionAndNavigate(test, uuid);
  await waitForObservedRequests(
      uuid, [group1ReportURL, createSellerReportURL(uuid)]);

  // Clear with an empty list, which should leave group 1 as well. Verify it can't
  // win an auction.
  await navigator.clearOriginJoinedAdInterestGroups(window.location.origin, []);
  await runBasicFledgeTestExpectingNoWinner(test, uuid);
}, 'clearOriginJoinedAdInterestGroups(), multiple groups joined, group list.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Join an interest group in a same-origin top-level window.
  await joinInterestGroupInTopLevelWindow(test, uuid, window.location.origin);

  // Make sure it was joined.
  await runBasicFledgeTestExpectingWinner(test, uuid);

  // Call "clearOriginJoinedAdInterestGroups()", which should leave the interest
  // group, since it was joined from a same-origin main frame.
  await navigator.clearOriginJoinedAdInterestGroups(window.location.origin);

  // Make sure group was left.
  await runBasicFledgeTestExpectingNoWinner(test, uuid);
}, 'clearOriginJoinedAdInterestGroups(), group joined from same-origin top-level context.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Create top-level browsing context for another origin, and have it join an
  // interest group owned by this document's origin.
  let topLevelWindow = await createTopLevelWindow(test, OTHER_ORIGIN1);
  let interestGroup = JSON.stringify(
      createInterestGroupForOrigin(uuid, window.location.origin));
  await runInFrame(test, topLevelWindow,
                   `await joinCrossOriginInterestGroup(test_instance, "${uuid}",
                                                       "${window.location.origin}",
                                                       ${interestGroup});`);

  // Call "clearOriginJoinedAdInterestGroups()", which should not leave the interest
  // group, since it was joined from a cross-origin main frame.
  await navigator.clearOriginJoinedAdInterestGroups(window.location.origin);

  // Make sure group was not left.
  await runBasicFledgeTestExpectingWinner(test, uuid);
}, 'clearOriginJoinedAdInterestGroups(), group joined from cross-origin top-level context.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinInterestGroup(test, uuid);

  // In a cross-origin iframe, call clearOriginJoinedAdInterestGroups() both for the
  // iframe's origin and for the main frame's origin. The latter should throw an
  // exception, and neither should manage to leave the interest group.
  let iframe = await createIframe(test, OTHER_ORIGIN1, 'join-ad-interest-group');
  await runInFrame(test, iframe,
                   `// Call clearOriginJoinedAdInterestGroups() with the iframe's origin.
                    await navigator.clearOriginJoinedAdInterestGroups(window.location.origin);
                    try {
                      // Call clearOriginJoinedAdInterestGroups() with the main frame's origin.
                      await navigator.clearOriginJoinedAdInterestGroups("${window.location.origin}");
                    } catch (e) {
                      assert_true(e instanceof DOMException, "DOMException thrown");
                      assert_equals(e.name, "NotAllowedError", "NotAllowedError DOMException thrown");
                      return {result: "success"};
                    }
                    throw "Exception unexpectedly not thrown";`);

  // Confirm that the interest group was not left.
  await runBasicFledgeTestExpectingWinner(test, uuid);
}, "clearOriginJoinedAdInterestGroups(), cross-origin iframe tries to leave parent frame's group.");

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // The possible results of calling clearOriginJoinedAdInterestGroups():

  // Doesn't throw an exception.
  const noExpectionURL = createTrackerURL(origin, uuid, "track_get", "no_exception");
  // Throws the exception it's expected to.
  const exceptionURL = createTrackerURL(origin, uuid, "track_get", "exception");
  // Throws the wrong exception.
  const badExpectionURL = createTrackerURL(origin, uuid, "track_get", "bad_exception");

  // Create a render URL that calls clearOriginJoinedAdInterestGroups() and
  // then requests one of the above tracking URLs, based on the resulting
  // behaviot.
  const renderURL = createRenderURL(
      uuid,
      `async function TryClear() {
         try {
           await navigator.clearOriginJoinedAdInterestGroups(
               "${window.location.origin}");
           await fetch("${noExpectionURL}");
         } catch (e) {
           if (e instanceof DOMException && e.name === "NotAllowedError") {
             await fetch("${exceptionURL}");
           } else {
             await fetch("${badExpectionURL}");
           }
         }
       }

       TryClear();`);

  await joinInterestGroup(
      test, uuid,
      {ads: [{ renderURL: renderURL}]});

  await runBasicFledgeAuctionAndNavigate(test, uuid);

  // This should wait until the clear call has thrown an exception.
  await waitForObservedRequests(
      uuid,
      [createBidderReportURL(uuid), createSellerReportURL(uuid), exceptionURL]);

  // Check the interest group was not left.
  await runBasicFledgeTestExpectingWinner(test, uuid);
}, 'clearOriginJoinedAdInterestGroups() in ad fenced frame throws an exception.');

///////////////////////////////////////////////////////////////////////////////
// Tests where some interest groups are owned by another origin.
///////////////////////////////////////////////////////////////////////////////

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Join interest group in iframe and make sure it was joined.
  let iframe = await createIframe(test, OTHER_ORIGIN1, 'join-ad-interest-group');
  await runInFrame(test, iframe,
                   `await joinInterestGroup(test_instance, "${uuid}");
                    await runBasicFledgeTestExpectingWinner(test_instance, "${uuid}");`);

  // In the main frame, Call clearOriginJoinedAdInterestGroups() for both the main
  // frame's origin, and the origin of the iframe / joined interest group. Neither
  // should leave the group, and the second should throw.
  await navigator.clearOriginJoinedAdInterestGroups(window.location.origin);
  try {
    await navigator.clearOriginJoinedAdInterestGroups(OTHER_ORIGIN1);
    throw 'Exception unexpectedly not thrown';
  } catch (e) {
    if (!(e instanceof DOMException) || e.name !== 'NotAllowedError') {
      throw 'Wrong exception thrown: ' + e.toString();
    }
  }

  // In an iframe, confirm the group was never left.
  await runInFrame(test, iframe,
      `await runBasicFledgeTestExpectingWinner(test_instance, "${uuid}");`);
}, 'clearOriginJoinedAdInterestGroups(). Cross-origin interest group joined in iframe, try to clear in main frame.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  let iframe = await createIframe(test, OTHER_ORIGIN1, 'join-ad-interest-group');
  await runInFrame(test, iframe,
                   `await joinInterestGroup(test_instance, "${uuid}");

                    // Confirm that trying to clear the interest group using the main frame's
                    // origin throws, and does not leave the group.
                    try {
                      await navigator.clearOriginJoinedAdInterestGroups("${window.location.origin}");
                      throw 'Exception unexpectedly not thrown';
                    } catch (e) {
                      if (!(e instanceof DOMException) || e.name !== 'NotAllowedError') {
                        throw 'Wrong exception thrown: ' + e.toString();
                      }
                    }
                    await runBasicFledgeTestExpectingWinner(test_instance, "${uuid}");`);
}, 'clearOriginJoinedAdInterestGroups(). Cross-origin interest group joined in iframe, clear call in iframe passing main frame origin.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  let iframe = await createIframe(test, OTHER_ORIGIN1, 'join-ad-interest-group');
  await runInFrame(test, iframe,
                   `await joinInterestGroup(test_instance, "${uuid}");

                    // Clear call with the origin of the cross-origin iframe.
                    // This should successfully leave the interest group.
                    await navigator.clearOriginJoinedAdInterestGroups("${OTHER_ORIGIN1}");

                    // Verify the group was left.
                    await runBasicFledgeTestExpectingNoWinner(test_instance, "${uuid}");`);
}, 'clearOriginJoinedAdInterestGroups(). Cross-origin interest group joined in iframe, clear call in iframe passing iframe origin.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Join an OTHER_ORIGIN1 interest group in an OTHER_ORIGIN1 main frame.
  let topLevelWindow = await createTopLevelWindow(test, OTHER_ORIGIN1);
  await runInFrame(test, topLevelWindow,
                   `await joinInterestGroup(test_instance, "${uuid}");`);

  let iframe = await createIframe(test, OTHER_ORIGIN1, 'join-ad-interest-group');

  await runInFrame(test, iframe,
                   `// Clear call from an OTHER_ORIGIN1 iframe on a different
                    // origin's main frame. This should not clear the interest
                    // group that was just joined, because the joining origin
                    // does not match.
                    await navigator.clearOriginJoinedAdInterestGroups("${OTHER_ORIGIN1}");

                    // Verify the group was not left.
                    await runBasicFledgeTestExpectingWinner(test_instance, "${uuid}");`);
}, 'clearOriginJoinedAdInterestGroups(). Cross-origin interest group joined from another joining origin, clear call in iframe.');
