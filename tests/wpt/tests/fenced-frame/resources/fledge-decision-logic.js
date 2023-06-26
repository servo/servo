// These functions are used by FLEDGE to determine the logic for the ad seller.
// For our testing purposes, we only need the minimal amount of boilerplate
// code in place to allow them to be invoked properly and move the FLEDGE
// process along. The tests do not deal with reporting results, so we leave
// `reportResult` empty. See `generateURNFromFledge` in "utils.js" to see how
// these files are used.

function scoreAd(
  adMetadata, bid, auctionConfig, trustedScoringSignals, browserSignals) {
  return 2*bid;
}

function reportResult(auctionConfig, browserSignals) {
  return;
}
