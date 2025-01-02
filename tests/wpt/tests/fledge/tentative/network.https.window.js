// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/subset-tests.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: timeout=long
// META: variant=?1-5
// META: variant=?6-10
// META: variant=?11-15
// META: variant=?16-last

"use strict";

// These tests focus on Protected Audience network requests - make sure they
// have no cookies, can set no cookies, and otherwise behave in the expected
// manner as related to the fetch spec. These tests don't cover additional
// request or response headers specific to Protected Audience API.

// URL that sets a cookie named "cookie" with a value of "cookie".
const SET_COOKIE_URL = `${BASE_URL}resources/set-cookie.asis`;

// URL that redirects to trusted bidding or scoring signals, depending on the
// query parameters, maintaining the query parameters for the redirect.
const REDIRECT_TO_TRUSTED_SIGNALS_URL = `${BASE_URL}resources/redirect-to-trusted-signals.py`;

// Returns a URL that stores request headers. Headers can later be retrieved
// as a name-to-list-of-values mapping with
// "(await fetchTrackedData(uuid)).trackedHeaders"
function createHeaderTrackerURL(uuid) {
  return createTrackerURL(window.location.origin, uuid, 'track_headers');
}

// Returns a URL that redirects to the provided URL. Uses query strings, so
// not suitable for generating trusted bidding/scoring signals URLs.
function createRedirectURL(location) {
  let url = new URL(`${BASE_URL}resources/redirect.py`);
  url.searchParams.append('location', location);
  return url.toString();
}

// Assert that "headers" has a single header with "name", whose value is "value".
function assertHasHeader(headers, name, value) {
  assert_equals(JSON.stringify(headers[name]), JSON.stringify([value]),
                'Header ' + name);
}

// Assert that "headers" has no header with "name"
function assertDoesNotHaveHeader(headers, name) {
  assert_equals(headers[name], undefined, 'Header ' + name);
}

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await setCookie(test);

  await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: { biddingLogicURL: createHeaderTrackerURL(uuid) }
      });

  let headers = (await fetchTrackedData(uuid)).trackedHeaders;
  assertHasHeader(headers, 'accept', 'application/javascript');
  assertHasHeader(headers, 'sec-fetch-dest', 'empty');
  assertHasHeader(headers, 'sec-fetch-mode', 'no-cors');
  assertHasHeader(headers, 'sec-fetch-site', 'same-origin');
  assertDoesNotHaveHeader(headers, 'cookie');
  assertDoesNotHaveHeader(headers, 'referer');
}, 'biddingLogicURL request headers.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await deleteAllCookies();

  await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: { biddingLogicURL: SET_COOKIE_URL }
      });

  assert_equals(document.cookie, '');
  await deleteAllCookies();
}, 'biddingLogicURL Set-Cookie.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: {
            biddingLogicURL: createRedirectURL(createBiddingScriptURL()) }
      });
}, 'biddingLogicURL redirect.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await setCookie(test);

  await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: { biddingWasmHelperURL: createHeaderTrackerURL(uuid) }
      });

  let headers = (await fetchTrackedData(uuid)).trackedHeaders;
  assertHasHeader(headers, 'accept', 'application/wasm');
  assertHasHeader(headers, 'sec-fetch-dest', 'empty');
  assertHasHeader(headers, 'sec-fetch-mode', 'no-cors');
  assertHasHeader(headers, 'sec-fetch-site', 'same-origin');
  assertDoesNotHaveHeader(headers, 'cookie');
  assertDoesNotHaveHeader(headers, 'referer');
}, 'biddingWasmHelperURL request headers.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await deleteAllCookies();

  await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: { biddingWasmHelperURL: SET_COOKIE_URL }
      });

  assert_equals(document.cookie, '');
  await deleteAllCookies();
}, 'biddingWasmHelperURL Set-Cookie.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides:
            { biddingWasmHelperURL: createRedirectURL(createBiddingWasmHelperURL()) }
      });
}, 'biddingWasmHelperURL redirect.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await setCookie(test);

  await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
      test,
      { uuid: uuid,
        auctionConfigOverrides: { decisionLogicURL: createHeaderTrackerURL(uuid) }
      });

  let headers = (await fetchTrackedData(uuid)).trackedHeaders;
  assertHasHeader(headers, 'accept', 'application/javascript');
  assertHasHeader(headers, 'sec-fetch-dest', 'empty');
  assertHasHeader(headers, 'sec-fetch-mode', 'no-cors');
  assertHasHeader(headers, 'sec-fetch-site', 'same-origin');
  assertDoesNotHaveHeader(headers, 'cookie');
  assertDoesNotHaveHeader(headers, 'referer');
}, 'decisionLogicURL request headers.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await deleteAllCookies();

  await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
      test,
      { uuid: uuid,
        auctionConfigOverrides: { decisionLogicURL: SET_COOKIE_URL }
      });

  assert_equals(document.cookie, '');
  await deleteAllCookies();
}, 'decisionLogicURL Set-Cookie.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
      test,
      { uuid: uuid,
        auctionConfigOverrides:
            { decisionLogicURL: createRedirectURL(createDecisionScriptURL(uuid)) }
      });
}, 'decisionLogicURL redirect.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await setCookie(test);

  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: {
          trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL,
          trustedBiddingSignalsKeys: ['headers'],
          biddingLogicURL: createBiddingScriptURL({
              generateBid:
                  `let headers = trustedBiddingSignals.headers;
                   function checkHeader(name, value) {
                     jsonActualValue = JSON.stringify(headers[name]);
                     if (jsonActualValue !== JSON.stringify([value]))
                       throw "Unexpected " + name + ": " + jsonActualValue;
                   }
                   checkHeader("accept", "application/json");
                   checkHeader("sec-fetch-dest", "empty");
                   checkHeader("sec-fetch-mode", "cors");
                   checkHeader("sec-fetch-site", "same-origin");
                   if (headers.cookie !== undefined)
                     throw "Unexpected cookie: " + JSON.stringify(headers.cookie);
                   if (headers.referer !== undefined)
                     throw "Unexpected referer: " + JSON.stringify(headers.referer);`,
          })
        }
      });
}, 'trustedBiddingSignalsURL request headers.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let cookieFrame = await createFrame(test, OTHER_ORIGIN1);
  await runInFrame(test, cookieFrame, `await setCookie(test_instance)`);

  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: {
          trustedBiddingSignalsURL: CROSS_ORIGIN_TRUSTED_BIDDING_SIGNALS_URL,
          trustedBiddingSignalsKeys: ['headers', 'cors'],
          biddingLogicURL: createBiddingScriptURL({
              generateBid:
                  `let headers = crossOriginTrustedBiddingSignals[
                       '${OTHER_ORIGIN1}'].headers;
                   function checkHeader(name, value) {
                     jsonActualValue = JSON.stringify(headers[name]);
                     if (jsonActualValue !== JSON.stringify([value]))
                       throw "Unexpected " + name + ": " + jsonActualValue;
                   }
                   checkHeader("accept", "application/json");
                   checkHeader("sec-fetch-dest", "empty");
                   checkHeader("sec-fetch-mode", "cors");
                   checkHeader("sec-fetch-site", "cross-site");
                   checkHeader("origin", "${window.location.origin}");
                   if (headers.cookie !== undefined)
                     throw "Unexpected cookie: " + JSON.stringify(headers.cookie);
                   if (headers.referer !== undefined)
                     throw "Unexpected referer: " + JSON.stringify(headers.referer);`,
          })
        }
      });
}, 'cross-origin trustedBiddingSignalsURL request headers.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await deleteAllCookies();

  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: { trustedBiddingSignalsURL: SET_COOKIE_URL }
      });

  assert_equals(document.cookie, '');
  await deleteAllCookies();
}, 'trustedBiddingSignalsURL Set-Cookie.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: {
          trustedBiddingSignalsURL: REDIRECT_TO_TRUSTED_SIGNALS_URL,
          trustedBiddingSignalsKeys: ['num-value'],
          biddingLogicURL: createBiddingScriptURL({
              generateBid:
                  `// The redirect should not be followed, so no signals should be received.
                   if (trustedBiddingSignals !== null)
                     throw "Unexpected trustedBiddingSignals: " + JSON.stringify(trustedBiddingSignals);`
          })
        }
      });
}, 'trustedBiddingSignalsURL redirect.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await setCookie(test);

  let renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'headers');

  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: {
          ads: [{ renderURL: renderURL }]
        },
        auctionConfigOverrides: {
          trustedScoringSignalsURL: TRUSTED_SCORING_SIGNALS_URL,
          decisionLogicURL: createDecisionScriptURL(uuid,
            {
              scoreAd:
                  `let headers = trustedScoringSignals.renderURL["${renderURL}"];
                   function checkHeader(name, value) {
                     jsonActualValue = JSON.stringify(headers[name]);
                     if (jsonActualValue !== JSON.stringify([value]))
                       throw "Unexpected " + name + ": " + jsonActualValue;
                   }
                   checkHeader("accept", "application/json");
                   checkHeader("sec-fetch-dest", "empty");
                   checkHeader("sec-fetch-mode", "cors");
                   checkHeader("sec-fetch-site", "same-origin");
                   if (headers.cookie !== undefined)
                     throw "Unexpected cookie: " + JSON.stringify(headers.cookie);
                   if (headers.referer !== undefined)
                     throw "Unexpected referer: " + JSON.stringify(headers.referer);`,
            })
        }
      });
}, 'trustedScoringSignalsURL request headers.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let cookieFrame = await createFrame(test, OTHER_ORIGIN1);
  await runInFrame(test, cookieFrame, `await setCookie(test_instance)`);

  let renderURL = createRenderURL(uuid, /*script=*/null, /*signalsParam=*/'headers,cors');

  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: {
          ads: [{ renderURL: renderURL }]
        },
        auctionConfigOverrides: {
          trustedScoringSignalsURL: CROSS_ORIGIN_TRUSTED_SCORING_SIGNALS_URL,
          decisionLogicURL: createDecisionScriptURL(uuid,
            {
              permitCrossOriginTrustedSignals: `"${OTHER_ORIGIN1}"`,
              scoreAd:
                  `let headers = crossOriginTrustedScoringSignals[
                      '${OTHER_ORIGIN1}'].renderURL["${renderURL}"];
                   function checkHeader(name, value) {
                     jsonActualValue = JSON.stringify(headers[name]);
                     if (jsonActualValue !== JSON.stringify([value]))
                       throw "Unexpected " + name + ": " + jsonActualValue;
                   }
                   checkHeader("accept", "application/json");
                   checkHeader("sec-fetch-dest", "empty");
                   checkHeader("sec-fetch-mode", "cors");
                   checkHeader("sec-fetch-site", "cross-site");
                   checkHeader("origin", "${window.location.origin}");
                   if (headers.cookie !== undefined)
                     throw "Unexpected cookie: " + JSON.stringify(headers.cookie);
                   if (headers.referer !== undefined)
                     throw "Unexpected referer: " + JSON.stringify(headers.referer);`,
            })
        }
      });
}, 'cross-origin trustedScoringSignalsURL request headers.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await deleteAllCookies();

  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test,
      { uuid: uuid,
        auctionConfigOverrides: { trustedScoringSignalsURL: SET_COOKIE_URL }
      });

  assert_equals(document.cookie, '');
  await deleteAllCookies();
}, 'trustedScoringSignalsURL Set-Cookie.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test,
      { uuid: uuid,
        auctionConfigOverrides: {
          trustedScoringSignalsURL: REDIRECT_TO_TRUSTED_SIGNALS_URL,
          decisionLogicURL: createDecisionScriptURL(uuid,
            {
              scoreAd:
                  `// The redirect should not be followed, so no signals should be received.
                   if (trustedScoringSignals !== null)
                     throw "Unexpected trustedScoringSignals: " + JSON.stringify(trustedScoringSignals);`
            })
        }
      });
}, 'trustedScoringSignalsURL redirect.');
