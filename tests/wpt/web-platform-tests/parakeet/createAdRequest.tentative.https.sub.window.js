// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

const okayAdRequest = {
  adRequestUrl: "https://{{host}}:{{ports[https][0]}}",
  adProperties: [
    { width: "42", height: "32", slot: "first", lang: "en-ca", adType: "test-ad1", bidFloor: 42.0 },
    { width: "24", height: "48", slot: "first", lang: "en-us", adType: "test-ad2", bidFloor: 42.0 }],
  publisherCode: "pubCode123",
  targeting: { interests: ["interest1", "interest2"] },
  anonymizedProxiedSignals: [],
  fallbackSource: "https://{{domains[www2]}}:{{ports[https][0]}}"
};

test(() => {
  assert_not_equals(navigator.createAdRequest, undefined);
}, "createAdRequest() should be supported on the navigator interface.");

promise_test(async t => {
  const createPromise = navigator.createAdRequest(okayAdRequest);

  await promise_rejects_dom(t, "NotSupportedError", createPromise);
}, "createAdRequest() should reject with NotSupported initially.");

promise_test(async t => {
  const createPromise = navigator.createAdRequest();

  await promise_rejects_js(t, TypeError, createPromise);
}, "createAdRequest() should reject missing parameters.");

promise_test(async t => {
  const adRequest = Object.assign({}, okayAdRequest);
  delete adRequest.adRequestUrl;

  const createPromise = navigator.createAdRequest(adRequest);

  await promise_rejects_js(t, TypeError, createPromise);
}, "createAdRequest() should reject a missing adRequestUrl.");

promise_test(async t => {
  const adRequest = Object.assign({}, okayAdRequest);
  adRequest.adRequestUrl = "http://{{host}}:{{ports[https][0]}}";

  const createPromise = navigator.createAdRequest(adRequest);

  await promise_rejects_js(t, TypeError, createPromise);
}, "createAdRequest() should reject a HTTP adRequestUrl.");

promise_test(async t => {
  const adRequest = Object.assign({}, okayAdRequest);
  delete adRequest.adProperties;

  const createPromise = navigator.createAdRequest(adRequest);

  await promise_rejects_js(t, TypeError, createPromise);
}, "createAdRequest() should reject missing adProperties.");

promise_test(async t => {
  const adRequest = Object.assign({}, okayAdRequest);
  adRequest.adProperties = [];

  const createPromise = navigator.createAdRequest(adRequest);

  await promise_rejects_js(t, TypeError, createPromise);
}, "createAdRequest() should reject empty adProperties.");

promise_test(async t => {
  const adRequest = Object.assign({}, okayAdRequest);
  adRequest.fallbackSource = "http://{{host}}:{{ports[https][0]}}";

  const createPromise = navigator.createAdRequest(adRequest);

  await promise_rejects_js(t, TypeError, createPromise);
}, "createAdRequest() should reject a HTTP fallbackSource.");

promise_test(async t => {
  const adRequest = Object.assign({}, okayAdRequest);

  // delete all optional params and the request should still be okay.
  delete adRequest.anonymizedProxiedSignals;
  delete adRequest.fallbackSource;
  delete adRequest.publisherCode;
  delete adRequest.targeting;

  const createPromise = navigator.createAdRequest(adRequest);

  // Until fully implemented we expect a NotSupportedError instead of success.
  await promise_rejects_dom(t, "NotSupportedError", createPromise);
}, "createAdRequest() should have optional params.");

promise_test(async t => {
  const adRequest = Object.assign({}, okayAdRequest);
  // A single adProperties object should be accepted as well as a sequence.
  adRequest.adProperties = { width: "24", height: "48", slot: "first", lang: "en-us", adType: "test-ad2", bidFloor: 42.0 };

  const createPromise = navigator.createAdRequest(adRequest);

  // Until fully implemented we expect a NotSupportedError instead of success.
  await promise_rejects_dom(t, "NotSupportedError", createPromise);
}, "createAdRequest() should accept a single adProperties.");

promise_test(async t => {
  const adRequest = Object.assign({}, okayAdRequest);
  adRequest.anonymizedProxiedSignals = ["coarse-geolocation", "coarse-ua", "targeting", "user-ad-interests"];

  const createPromise = navigator.createAdRequest(adRequest);

  // Until fully implemented we expect a NotSupportedError instead of success.
  await promise_rejects_dom(t, "NotSupportedError", createPromise);
}, "createAdRequest() should accept valid anonymizedProxiedSignals.");

promise_test(async t => {
  const adRequest = Object.assign({}, okayAdRequest);
  adRequest.anonymizedProxiedSignals = ["coarse-geolocation", "unknown-type"];

  const createPromise = navigator.createAdRequest(adRequest);

  await promise_rejects_js(t, TypeError, createPromise);
}, "createAdRequest() should reject unknown anonymizedPRoxiedSignals.");