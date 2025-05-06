// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js

"use strict";

subsetTest(
    promise_test,
    async test => {
      const uuid = generateUuid(test);

      let reportWin = `
      if (browserSignals.kAnonStatus !== "belowThreshold") {
        sendReportTo('${createBidderReportURL(uuid, 'error')}');
        return false;
      }
      sendReportTo('${createBidderReportURL(uuid)}');
    `;
      let interestGroupOverrides = {
        biddingLogicURL: createBiddingScriptURL({reportWin: reportWin})
      };
      let interestGroup = createInterestGroupForOrigin(
          uuid, window.location.origin, interestGroupOverrides);
      await joinInterestGroupWithoutDefaults(test, interestGroup);

      // Make the interest group not k-anonymous.
      await test_driver.set_protected_audience_k_anonymity(
          interestGroup.owner, interestGroup.name, []);

      let auctionConfigOverrides = {
        decisionLogicURL: createDecisionScriptURL(uuid, {})
      };
      await runBasicFledgeAuctionAndNavigate(
          test, uuid, auctionConfigOverrides);
      await waitForObservedRequests(uuid, [createBidderReportURL(uuid)]);
    },
    'Check kAnonStatus is "belowThreshold" when FledgeConsiderKAnonymity' +
        'is enabled and FledgeEnforceKAnonymity is disabled');
