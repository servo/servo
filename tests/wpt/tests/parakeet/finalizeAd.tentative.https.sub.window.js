// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

const okayAuctionRequest = {
  seller: "https://{{host}}:{{ports[https][0]}}",
  decisionLogicUrl: "https://{{host}}:{{ports[https][0]}}",
  perBuyerSignals: {"{{host}}": { randomParam: "value1" }},
  auctionSignals: "pubCode123",
  sellerSignals: { someKey: "sellerValue" }
};

test(() => {
  assert_not_equals(navigator.finalizeAd, undefined);
}, "finalizeAd() should be supported on the navigator interface.");

promise_test(async t => {
  const finalizePromise = navigator.finalizeAd({}, okayAuctionRequest);

  await promise_rejects_js(t, TypeError, finalizePromise);
}, "finalizeAd() should reject an invalid Ads object.");

promise_test(async t => {
  const auctionRequest = Object.assign({}, okayAuctionRequest);
  delete auctionRequest.decisionLogicUrl;

  const finalizePromise = navigator.finalizeAd({}, auctionRequest);

  await promise_rejects_js(t, TypeError, finalizePromise);
}, "finalizeAd() should reject a missing decisionLogicUrl.");

promise_test(async t => {
  const auctionRequest = Object.assign({}, okayAuctionRequest);
  auctionRequest.decisionLogicUrl = "http://{{host}}:{{ports[https][0]}}";

  const finalizePromise = navigator.finalizeAd({}, auctionRequest);

  await promise_rejects_js(t, TypeError, finalizePromise);
}, "finalizeAd() should reject a non-HTTPS decisionLogicUrl.");