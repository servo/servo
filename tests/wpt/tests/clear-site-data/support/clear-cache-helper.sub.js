/* Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/publicdomain/zero/1.0/ */

"use strict"

const sameOrigin =
  'https://{{host}}:{{ports[https][0]}}';
const subdomainOrigin =
  'https://{{hosts[][www2]}}:{{ports[https][0]}}';
const crossSiteOrigin =
  'https://{{hosts[alt][]}}:{{ports[https][0]}}';
const subomdainCrossSiteOrigin =
  'https://{{hosts[alt][www2]}}:{{ports[https][0]}}';

/**
 * Constructs a url for an intermediate "bounce" hop which represents a tracker.
 * @param {string} cacheHelper - Unique uuid for this test
 * @param {*} options - URL generation options.
 * @param {boolean} [options.second_origin = true] - whether domain should be a different origin
 * @param {boolean} [options.subdomain = false] - whether the domain should start with
 *        a different subdomain
 * @param {boolean} [options.cache = false] - whether the resource should be cacheable
 * @param {(null|'cache'|'all')} [options.clear] - whether to send the
 *        Clear-Site-Data header.
 * @param {(null|'cache'|'all')} [options.clear_first] - whether to send the
 *        Clear-Site-Data header on first response
 * @param {string} [response] - which response to elict - defaults to "single_html". Other
 *        options can be found in "clear-site-data-cache.py" server helper.
 * @param {*} [options.iframe] - iframe same parameters as options (recursive). Only works on
 *        "single_html" variation of response
 */
function getUrl(cacheHelper, {
    subdomain = false,
    secondOrigin = false,
    cache = false,
    clear = null,
    clearFirst = null,
    response = "single_html",
    iframe = null,
}) {
    let url = "https://";
    if (subdomain && secondOrigin) {
        url += "{{hosts[alt][www2]}}";
    } else if (subdomain) { // && !second_origin
        url += "{{hosts[][www2]}}";
    } else if (secondOrigin) { // && !subdomain
        url += "{{hosts[alt][]}}";
    } else { // !second_origin && !subdomain
        url += "{{hosts[][]}}";
    }
    url += ":{{ports[https][0]}}";
    url += "/clear-site-data/support/clear-site-data-cache.py";
    url = new URL(url);
    let params = new URLSearchParams();
    params.append("cache_helper", cacheHelper);
    params.append("response", response)
    if (clear !== null) {
        params.append("clear", clear);
    }
    if (clearFirst != null) {
        params.append("clear_first", clearFirst);
    }
    if (cache) {
        params.append("cache", "");
    }
    if (iframe != null) {
        let iframeUrl = getUrl(cacheHelper, iframe);
        params.append("iframe", iframeUrl);
    }
    url.search = params;
    return url.toString();
}

/**
 * Opens test pages sequentially, compares first and last uuid. Makes sure test cleans up properly
 * @param test - test clean up
 * @param {string} firstUuid - uuid returned by first url
 * @param {array[string]} testUrls - array of all urls that should be visited
 * @param {integer} curIdx - index in testUrls that is visited in the current function call
 * @param {function assert_not_equal|assert_equal} assert - function that gets passed first and last
 *        uuid and determines the success of the test case
 * @param {function} resolve - function to call when test case is complete
 * @param {*} options - URL generation options.
 */
function openTestPageHelper(test, firstUuid, testUrls, curIdx, assert, resolve) {
    window.addEventListener("message", test.step_func(e => {
        let curUuid = e.data;
        if (firstUuid === null) {
            firstUuid = curUuid;
        }

        if (curIdx + 1 < testUrls.length) {
            openTestPageHelper(test, firstUuid, testUrls, curIdx + 1, assert, resolve);
        } else {
            // Last Step
            assert(firstUuid, curUuid);
            resolve();
        }
    }), {once: true});

    window.open(testUrls[curIdx]);
}

// Here's the set-up for this test: Step 1 and Step 2 are repeated for each param in params
// Step 1 (main window) Open popup window with url generated with `getUrl`
// Step 2 (first window) Message main window with potentially cached uuid and close popup
// Last Step (main window): Assert first and last uuid not equal due to `clear-site-data: "cache"` header
//
// Basic diagram visualizing how the test works:
//
//     main window opens sequentially:
//             (1)                  (2)                (last) = (1)
//              | Step 1             | Step 3                | Step 4
//              |                    |                       |
//     +--------v---------+   +------v----------+     +------v-----------+
//     | first / second   |   |  Clear Data?    |     |                  |
//     | origin           |   |                 |     |                  |
//     |                  |   |                 |     |                  |
//     | +-iframe-------+ |   | +-(iframe?)---+ | ... | +-iframe-------+ |
//     | | first/second | |   | | Clear Data? | |     | |              | |
//     | | origin       | |   | |             | |     | |              | |
//     | +-----------+--+ |   | +-------------+ |     | +-+------------+ |
//     +-------------+----+   +-----------------+     +---+--------------+
//                   |                                    |
//                   | Step 2            +----------------+ Step 5
//                   |                   |
//                   v                   v
//     Last Step: is uuid from (1) different from (last)?
function testCacheClear(test, params, assert) {
    if (params.length < 2) {
        // fail test case
        return new Promise((resolve, reject) => reject());
    }

    const cacheHelper = self.crypto.randomUUID();
    const testUrls = params.map((param) => getUrl(cacheHelper, param));

    return new Promise(resolve => {
        openTestPageHelper(test, null, testUrls, 0, assert, resolve)
    });
}

// The tests are built on top of the back-forward-cache test harness.
// Here is the steps for the tests:
// 1. Open a new window and navigate to a test URL.
// 2. Navigate the window to a second page.
// 3. Trigger the clear-site-data header either by window.open() or loading an
//    iframe from the second page.
// 4. Navigate back to the first page.
// 5. Assert that the first page is or is not cached.

function runBfCacheClearTest(params, description) {
  runBfcacheTest(
    {
      targetOrigin: sameOrigin,
      scripts: ["/clear-site-data/support/clear-cache-helper.sub.js"],
      funcBeforeBackNavigation: async (getUrlParams, mode) => {

        const cacheHelper = self.crypto.randomUUID();
        const testUrl = getUrl(cacheHelper, getUrlParams);

        let clearingPromise;
        if (mode === "window") {
          clearingPromise = new Promise(resolve => {
            window.addEventListener("message", resolve, {once: true});
            window.open(testUrl);
          });
        } else if (mode === "iframe") {
          clearingPromise = new Promise(resolve => {
            const iframe = document.createElement("iframe");
            iframe.src = testUrl;
            document.body.appendChild(iframe);
            iframe.onload = resolve;
          });
        } else {
          throw new Error("Unsupported mode");
        }

        await clearingPromise;
      },
      argsBeforeBackNavigation: [params.getUrlParams, params.mode],
      ...params,
    },
    description
  );
}

