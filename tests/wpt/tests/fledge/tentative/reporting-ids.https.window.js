// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-5
// META: variant=?6-last


"use strict;"

const makeTest = ({
  // Name of the test.
  name,
  // The value of the selectableReportingIds to be used in the test.
  selectableBuyerAndSellerReportingIds = undefined,
  // The value of the buyerAndSellerReportingId to be used in the test.
  buyerAndSellerReportingId = undefined,
  // The value of the buyerReportingId to be used in the test.
  buyerReportingId = undefined,
  // The javascript comparison to be used in generateBid()
  generateBidComparison,
  // what gets returned by generateBid()
  generateBidReturn,
  // The javascript comparison to be used in scoreAd()
  scoreAdComparison,
  // The javascript comparison to be used in reportWin()
  reportWinComparison,
  // The javascript comparison to be used in reportResult()
  reportResultComparison,
  // Whether we expect the auction to have a winner or not.
  expectWinner = true,
}) => {
  subsetTest(promise_test, async test => {
    const uuid = generateUuid(test);
    let renderURL = createRenderURL(uuid);
    const bidderReportURL = createBidderReportURL(uuid);
    const sellerReportURL = createSellerReportURL(uuid);

    let interestGroupOverrides = {};
    interestGroupOverrides.ads = [{
      renderURL: renderURL,
      selectableBuyerAndSellerReportingIds: selectableBuyerAndSellerReportingIds,
      buyerAndSellerReportingId: buyerAndSellerReportingId,
      buyerReportingId: buyerReportingId,
    }];
    interestGroupOverrides.biddingLogicURL = createBiddingScriptURL(
      {
        generateBid:
          `
          if(${(generateBidComparison)}){
            return ${generateBidReturn};
          }
          throw "Failed comparison in generateBid: " + interestGroup["ads"][0];
          `,
        reportWin:
          `
          if(${reportWinComparison})
            sendReportTo('${bidderReportURL}')
          else
            throw "Failed comparison in reportWin";
          `
      });

    let auctionConfigOverrides = {};
    auctionConfigOverrides.decisionLogicURL = createDecisionScriptURL(
      uuid,
      {
        scoreAd:
          `
          if (${scoreAdComparison}){
            return {desirability: 10, bid: 10}
           }
           throw "Failed comparison in scoreAd ";
          `,
        reportResult:
          `
          if(${reportResultComparison})
              sendReportTo("${sellerReportURL}");
          else
            throw "Failed comparison in reportResult";
          `
      });

    await joinInterestGroup(test, uuid, interestGroupOverrides);

    if (expectWinner) {
      await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfigOverrides);
      await waitForObservedRequests(
        uuid,
        [bidderReportURL, sellerReportURL]);
    }
    else {
      await runBasicFledgeTestExpectingNoWinner(test, uuid, auctionConfigOverrides);
    }
  }, name);
};

// Verify that basic form of selectableReportingIds and selectedReportingId are where they should be.
makeTest({
  name: "selectableBuyerAndSellerReportingIds",
  selectableBuyerAndSellerReportingIds: ["selectable_id1", "selectable_id2"],
  generateBidComparison: `deepEquals(interestGroup["ads"][0].selectableBuyerAndSellerReportingIds, ["selectable_id1", "selectable_id2"])`,
  generateBidReturn: `{bid:10, render: interestGroup.ads[0].renderURL,
                      selectedBuyerAndSellerReportingId:interestGroup.ads[0].selectableBuyerAndSellerReportingIds[0]}`,
  scoreAdComparison: `browserSignals.selectedBuyerAndSellerReportingId === "selectable_id1" &&
                      browserSignals.buyerAndSellerReportingId === undefined &&
                      browserSignals.buyerReportingId === undefined`,
  reportWinComparison: `browserSignals.selectedBuyerAndSellerReportingId === "selectable_id1" &&
                      browserSignals.buyerAndSellerReportingId === undefined &&
                      browserSignals.buyerReportingId === undefined`,
  reportResultComparison: `browserSignals.selectedBuyerAndSellerReportingId === "selectable_id1" &&
                         browserSignals.buyerAndSellerReportingId === undefined &&
                         browserSignals.buyerReportingId === undefined`,
});

// Verify the buyer and seller reporting id is where we expect it when it is present alongside the selected reporting id.
makeTest({
  name: "selectableBuyerAndSellerReportingIds and buyerAndSellerReportingId",
  selectableBuyerAndSellerReportingIds: ["selectable_id1", "selectable_id2"],
  buyerAndSellerReportingId: "buyer_and_seller_reporting_id",
  generateBidComparison: `deepEquals(interestGroup["ads"][0].selectableBuyerAndSellerReportingIds, ["selectable_id1", "selectable_id2"]) &&
                          interestGroup["ads"][0].buyerAndSellerReportingId === "buyer_and_seller_reporting_id"`,
  generateBidReturn: `{bid:10, render: interestGroup.ads[0].renderURL,
                      selectedBuyerAndSellerReportingId:interestGroup.ads[0].selectableBuyerAndSellerReportingIds[0]}`,
  scoreAdComparison: `browserSignals.selectedBuyerAndSellerReportingId === "selectable_id1" &&
                      browserSignals.buyerAndSellerReportingId === "buyer_and_seller_reporting_id" &&
                      browserSignals.buyerReportingId === undefined`,
  reportWinComparison: `browserSignals.selectedBuyerAndSellerReportingId === "selectable_id1" &&
                        browserSignals.buyerAndSellerReportingId === "buyer_and_seller_reporting_id" &&
                        browserSignals.buyerReportingId === undefined`,
  reportResultComparison: `browserSignals.selectedBuyerAndSellerReportingId === "selectable_id1" &&
                           browserSignals.buyerAndSellerReportingId === "buyer_and_seller_reporting_id" &&
                           browserSignals.buyerReportingId === undefined`,
});

// Verify the buyer reporting id is where we expect it when it is present alongside the selected reporting id.
makeTest({
  name: "selectableBuyerAndSellerReportingIds and buyerReportingId",
  selectableBuyerAndSellerReportingIds: ["selectable_id1", "selectable_id2"],
  buyerReportingId: "buyer_reporting_id",
  generateBidComparison: `deepEquals(interestGroup["ads"][0].selectableBuyerAndSellerReportingIds, ["selectable_id1", "selectable_id2"]) &&
                          interestGroup["ads"][0].buyerReportingId === "buyer_reporting_id"`,
  generateBidReturn: `{bid:10, render: interestGroup.ads[0].renderURL,
                      selectedBuyerAndSellerReportingId:interestGroup.ads[0].selectableBuyerAndSellerReportingIds[0]}`,
  scoreAdComparison: `browserSignals.selectedBuyerAndSellerReportingId === "selectable_id1" &&
                      browserSignals.buyerAndSellerReportingId === undefined &&
                      browserSignals.buyerReportingId === undefined`,
  reportWinComparison: `browserSignals.selectedBuyerAndSellerReportingId === "selectable_id1" &&
                        browserSignals.buyerAndSellerReportingId === undefined &&
                        browserSignals.buyerReportingId === "buyer_reporting_id"`,
  reportResultComparison: `browserSignals.selectedBuyerAndSellerReportingId === "selectable_id1" &&
                           browserSignals.buyerAndSellerReportingId === undefined &&
                           browserSignals.buyerReportingId === undefined`,
});

// Verify all reporting ids are where we expect when they are all present alongside the selected reporting id.
makeTest({
  name: "selectableBuyerAndSellerReportingIds, buyerAndSellerReportingId, and buyerReportingId",
  selectableBuyerAndSellerReportingIds: ["selectable_id1", "selectable_id2"],
  buyerAndSellerReportingId: "buyer_and_seller_reporting_id",
  buyerReportingId: "buyer_reporting_id",
  generateBidComparison: `deepEquals(interestGroup["ads"][0].selectableBuyerAndSellerReportingIds, ["selectable_id1", "selectable_id2"]) &&
                          interestGroup["ads"][0].buyerAndSellerReportingId === "buyer_and_seller_reporting_id" &&
                          interestGroup["ads"][0].buyerReportingId === "buyer_reporting_id"`,
  generateBidReturn: `{bid:10, render: interestGroup.ads[0].renderURL,
                       selectedBuyerAndSellerReportingId:interestGroup.ads[0].selectableBuyerAndSellerReportingIds[0]}`,
  scoreAdComparison: `browserSignals.selectedBuyerAndSellerReportingId === "selectable_id1" &&
                      browserSignals.buyerAndSellerReportingId === "buyer_and_seller_reporting_id" &&
                      browserSignals.buyerReportingId === undefined`,
  reportWinComparison: `browserSignals.selectedBuyerAndSellerReportingId === "selectable_id1" &&
                        browserSignals.buyerAndSellerReportingId === "buyer_and_seller_reporting_id" &&
                        browserSignals.buyerReportingId === "buyer_reporting_id"`,
  reportResultComparison: `browserSignals.selectedBuyerAndSellerReportingId === "selectable_id1" &&
                           browserSignals.buyerAndSellerReportingId === "buyer_and_seller_reporting_id" &&
                           browserSignals.buyerReportingId === undefined`,
});

// Verify old behavior occurs when no id was selected, even though there was selectable ids.
makeTest({
  name: "selectableBuyerAndSellerReportingIds but none selected, buyerAndSellerReportingId, and buyerReportingId",
  selectableBuyerAndSellerReportingIds: ["selectable_id1", "selectable_id2"],
  buyerAndSellerReportingId: "buyer_and_seller_reporting_id",
  buyerReportingId: "buyer_reporting_id",
  generateBidComparison: `deepEquals(interestGroup["ads"][0].selectableBuyerAndSellerReportingIds, ["selectable_id1", "selectable_id2"]) &&
                          interestGroup["ads"][0].buyerAndSellerReportingId === "buyer_and_seller_reporting_id" &&
                          interestGroup["ads"][0].buyerReportingId === "buyer_reporting_id"`,
  generateBidReturn: `{bid:10, render: interestGroup.ads[0].renderURL}`,
  scoreAdComparison: `browserSignals.selectedBuyerAndSellerReportingId === undefined &&
                      browserSignals.buyerAndSellerReportingId === undefined &&
                      browserSignals.buyerReportingId === undefined`,
  reportWinComparison: `browserSignals.selectedBuyerAndSellerReportingId === undefined &&
                        browserSignals.buyerAndSellerReportingId === "buyer_and_seller_reporting_id" &&
                        browserSignals.buyerReportingId === undefined`,
  reportResultComparison: `browserSignals.selectedBuyerAndSellerReportingId === undefined &&
                           browserSignals.buyerAndSellerReportingId === "buyer_and_seller_reporting_id" &&
                           browserSignals.buyerReportingId === undefined`,
});

// Make sure if there was a selectedReportingId that's not within the selectables,
// that the worklet throws an error and the auction does not have a winner.
makeTest({
  name: "selected not included selectables",
  selectableBuyerAndSellerReportingIds: ["selectable_id1", "selectable_id2"],
  buyerAndSellerReportingId: "buyer_and_seller_reporting_id",
  buyerReportingId: "buyer_reporting_id",
  generateBidComparison: `deepEquals(interestGroup["ads"][0].selectableBuyerAndSellerReportingIds, ["selectable_id1", "selectable_id2"]) &&
                          interestGroup["ads"][0].buyerAndSellerReportingId === "buyer_and_seller_reporting_id" &&
                          interestGroup["ads"][0].buyerReportingId === "buyer_reporting_id"`,
  generateBidReturn: `{bid:10, render: interestGroup.ads[0].renderURL, selectedBuyerAndSellerReportingId:"invalid_id"}`,
  scoreAdComparison: `browserSignals.selectedBuyerAndSellerReportingId === undefined &&
                      browserSignals.buyerAndSellerReportingId === undefined &&
                      browserSignals.buyerReportingId === undefined`,
  reportWinComparison: `false`,
  reportResultComparison: `false"`,
  expectWinner: false
});

// Make sure if there was a selectedReportingId but no selectables,
// that the worklet throws an error and the auction does not have a winner
makeTest({
  name: "selected without selectables",
  buyerAndSellerReportingId: "buyer_and_seller_reporting_id",
  buyerReportingId: "buyer_reporting_id",
  generateBidComparison: `interestGroup["ads"][0].selectableBuyerAndSellerReportingIds === undefined &&
                          interestGroup["ads"][0].buyerAndSellerReportingId === undefined &&
                          interestGroup["ads"][0].buyerReportingId === undefined`,
  generateBidReturn: `{bid:10, render: interestGroup.ads[0].renderURL, selectedBuyerAndSellerReportingId:"invalid_id"}`,
  scoreAdComparison: `browserSignals.selectedBuyerAndSellerReportingId === undefined &&
                      browserSignals.buyerAndSellerReportingId === undefined &&
                      browserSignals.buyerReportingId === undefined`,
  reportWinComparison: `false`,
  reportResultComparison: `false`,
  expectWinner: false
});

// Verify old behavior occurs when there are no present selectable ids.
makeTest({
  name: "buyerAndSellerReportingId, and buyerReportingId, but no selectables present",
  buyerAndSellerReportingId: "buyer_and_seller_reporting_id",
  buyerReportingId: "buyer_reporting_id",
  generateBidComparison: `interestGroup["ads"][0].selectableBuyerAndSellerReportingIds === undefined &&
                          interestGroup["ads"][0].buyerAndSellerReportingId === undefined &&
                          interestGroup["ads"][0].buyerReportingId === undefined`,
  generateBidReturn: `{bid:10, render: interestGroup.ads[0].renderURL}`,
  scoreAdComparison: `browserSignals.selectedBuyerAndSellerReportingId === undefined &&
                      browserSignals.buyerAndSellerReportingId === undefined &&
                      browserSignals.buyerReportingId === undefined`,
  reportWinComparison: `browserSignals.selectedBuyerAndSellerReportingId === undefined &&
                        browserSignals.buyerAndSellerReportingId === "buyer_and_seller_reporting_id" &&
                        browserSignals.buyerReportingId === undefined`,
  reportResultComparison: `browserSignals.selectedBuyerAndSellerReportingId === undefined &&
                           browserSignals.buyerAndSellerReportingId === "buyer_and_seller_reporting_id" &&
                           browserSignals.buyerReportingId === undefined`,
});

makeTest({
  name: "only buyerAndSellerReportingId",
  buyerAndSellerReportingId: "buyer_and_seller_reporting_id",
  generateBidComparison: `interestGroup["ads"][0].selectableBuyerAndSellerReportingIds === undefined &&
                          interestGroup["ads"][0].buyerAndSellerReportingId === undefined &&
                          interestGroup["ads"][0].buyerReportingId === undefined`,
  generateBidReturn: `{bid:10, render: interestGroup.ads[0].renderURL}`,
  scoreAdComparison: `browserSignals.selectedBuyerAndSellerReportingId === undefined &&
                      browserSignals.buyerAndSellerReportingId === undefined &&
                      browserSignals.buyerReportingId === undefined`,
  reportWinComparison: `browserSignals.selectedBuyerAndSellerReportingId === undefined &&
                        browserSignals.buyerAndSellerReportingId === "buyer_and_seller_reporting_id" &&
                        browserSignals.buyerReportingId === undefined`,
  reportResultComparison: `browserSignals.selectedBuyerAndSellerReportingId === undefined &&
                           browserSignals.buyerAndSellerReportingId === "buyer_and_seller_reporting_id" &&
                           browserSignals.buyerReportingId === undefined`,
});

makeTest({
  name: "only buyerReportingId",
  buyerReportingId: "buyer_reporting_id",
  generateBidComparison: `interestGroup["ads"][0].selectableBuyerAndSellerReportingIds === undefined &&
                          interestGroup["ads"][0].buyerAndSellerReportingId === undefined &&
                          interestGroup["ads"][0].buyerReportingId === undefined`,
  generateBidReturn: `{bid:10, render: interestGroup.ads[0].renderURL}`,
  scoreAdComparison: `browserSignals.selectedBuyerAndSellerReportingId === undefined &&
                      browserSignals.buyerAndSellerReportingId === undefined &&
                      browserSignals.buyerReportingId === undefined`,
  reportWinComparison: `browserSignals.selectedBuyerAndSellerReportingId === undefined &&
                        browserSignals.buyerAndSellerReportingId === undefined &&
                        browserSignals.buyerReportingId === "buyer_reporting_id"`,
  reportResultComparison: `browserSignals.selectedBuyerAndSellerReportingId === undefined &&
                           browserSignals.buyerAndSellerReportingId === undefined &&
                           browserSignals.buyerReportingId === undefined`,
});

makeTest({
  name: "no reporting ids, expect IG name",
  generateBidComparison: `interestGroup["ads"][0].selectableBuyerAndSellerReportingIds === undefined &&
                          interestGroup["ads"][0].buyerAndSellerReportingId === undefined &&
                          interestGroup["ads"][0].buyerReportingId === undefined`,
  generateBidReturn: `{bid:10, render: interestGroup.ads[0].renderURL}`,
  scoreAdComparison: `browserSignals.selectedBuyerAndSellerReportingId === undefined &&
                      browserSignals.buyerAndSellerReportingId === undefined &&
                      browserSignals.buyerReportingId === undefined`,
  reportWinComparison: `browserSignals.selectedBuyerAndSellerReportingId === undefined &&
                        browserSignals.buyerAndSellerReportingId === undefined &&
                        browserSignals.buyerReportingId === undefined &&
                        browserSignals.interestGroupName === "default name"`,
  reportResultComparison: `browserSignals.selectedBuyerAndSellerReportingId === undefined &&
                           browserSignals.buyerAndSellerReportingId === undefined &&
                           browserSignals.buyerReportingId === undefined`,
});
