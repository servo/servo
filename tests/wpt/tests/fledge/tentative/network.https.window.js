// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/subset-tests.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: timeout=long
// META: variant=?1-5
// META: variant=?6-last

"use strict";

// These tests focus on Protected Audience network requests - make sure they
// have no cookies, can set no cookies, and otherwise behave in the expected
// manner as related to the fetch spec. These tests don't cover additional
// request or response headers specific to Protected Audience API.

// URL that sets a cookie named "cookie" with a value of "cookie".
const SET_COOKIE_URL = `${BASE_URL}resources/set-cookie.asis`;

// Returns a URL that stores request headers. Headers can later be retrieved
// as a name-to-list-of-values mapping with
// "(await fetchTrackedData(uuid)).trackedHeaders"
function createHeaderTrackerURL(uuid) {
  return createTrackerURL(window.location.origin, uuid, 'track_headers');
}

// Delete all cookies. Separate function so that can be replaced with
// something else for testing outside of a WPT environment.
async function deleteAllCookies() {
  await test_driver.delete_all_cookies();
}

// Deletes all cookies (to avoid pre-existing cookies causing inconsistent
// output on failure) and sets a cookie with name "cookie" and a value of
// "cookie". Adds a cleanup task to delete all cookies again when the test
// is done.
async function setCookie(test) {
  await deleteAllCookies();
  document.cookie = 'cookie=cookie; path=/'
  test.add_cleanup(deleteAllCookies);
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
  await deleteAllCookies();

  await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
      test,
      { uuid: uuid,
        interestGroupOverrides: { biddingLogicURL: SET_COOKIE_URL }
      });

  assert_equals(document.cookie, '');
  await deleteAllCookies();
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
                   checkHeader("sec-fetch-mode", "no-cors");
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
                   checkHeader("sec-fetch-mode", "no-cors");
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
  await deleteAllCookies();

  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test,
      { uuid: uuid,
        auctionConfigOverrides: { trustedScoringSignalsURL: SET_COOKIE_URL }
      });

  assert_equals(document.cookie, '');
  await deleteAllCookies();
}, 'trustedScoringSignalsURL Set-Cookie.');
