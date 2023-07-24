// These functions are used by FLEDGE to determine the logic for the ad buyer.
// For our testing purposes, we only need the minimal amount of boilerplate
// code in place to allow them to be invoked properly and move the FLEDGE
// process along. The tests do not deal with reporting results, so we leave
// `reportWin` empty. See `generateURNFromFledge` in "utils.js" to see how
// these files are used.

function generateBid(interestGroup, auctionSignals, perBuyerSignals,
  trustedBiddingSignals, browserSignals) {
  const ad = interestGroup.ads[0];

  // `auctionSignals` controls whether or not component auctions are allowed.
  let allowComponentAuction =
    typeof auctionSignals === 'string' &&
    auctionSignals.includes('bidderAllowsComponentAuction');

  let result = {
    'ad': ad,
    'bid': 1,
    'render': { url: ad.renderUrl, width: "100px", height: "50px" },
    'allowComponentAuction': allowComponentAuction
  };
  if (interestGroup.adComponents && interestGroup.adComponents.length > 0)
    result.adComponents = interestGroup.adComponents.map((component) => {
      return {
        url: component.renderUrl,
        width: "100px",
        height: "50px"
      };
    });
  return result;
}

function reportWin(
  auctionSignals, perBuyerSignals, sellerSignals, browserSignals) {
  return;
}
