"use strict";

const SAME_ORIGIN = "https://{{host}}:{{ports[h2][0]}}";
const CROSS_ORIGIN = "https://{{hosts[alt][www]}}:{{ports[h2][0]}}";

const RESOURCES_PATH = "/loading/early-hints/resources";
const SAME_ORIGIN_RESOURCES_URL = SAME_ORIGIN + RESOURCES_PATH;
const CROSS_ORIGIN_RESOURCES_URL = CROSS_ORIGIN + RESOURCES_PATH;

/**
 * Navigate to a test page with an Early Hints response.
 *
 * @typedef {Object} Preload
 * @property {string} url - A URL to preload. Note: This is relative to the
 *     `test_url` parameter of `navigateToTestWithEarlyHints()`.
 * @property {string} as_attr - `as` attribute of this preload.
 * @property {string} [crossorigin_attr] - `crossorigin` attribute of this
 *     preload.
 * @property {string} [fetchpriority_attr] - `fetchpriority` attribute of this
 *     preload.
 *
 * @param {string} test_url - URL of a test after the Early Hints response.
 * @param {Array<Preload>} preloads  - Preloads included in the Early Hints response.
 * @param {bool} exclude_preloads_from_ok_response - Whether to exclude the preloads from the 200 OK reponse.
 */
function navigateToTestWithEarlyHints(test_url, preloads, exclude_preloads_from_ok_response) {
    const params = new URLSearchParams();
    params.set("test_url", test_url);
    params.set("exclude_preloads_from_ok_response",
               (!!exclude_preloads_from_ok_response).toString());
    for (const preload of preloads) {
        params.append("preloads", JSON.stringify(preload));
    }
    const url = RESOURCES_PATH +"/early-hints-test-loader.h2.py?" + params.toString();
    window.location.replace(new URL(url, window.location));
}

/**
 * Parses the query string of the current window location and returns preloads
 * in the Early Hints response sent via `navigateToTestWithEarlyHints()`.
 *
 * @returns {Array<Preload>}
 */
function getPreloadsFromSearchParams() {
    const params = new URLSearchParams(window.location.search);
    const encoded_preloads = params.getAll("preloads");
    const preloads = [];
    for (const encoded of encoded_preloads) {
        preloads.push(JSON.parse(encoded));
    }
    return preloads;
}

/**
 * Fetches a script or an image.
 *
 * @param {string} element - "script" or "img".
 * @param {string} url - URL of the resource.
 */
async function fetchResource(element, url) {
    return new Promise((resolve, reject) => {
        const el = document.createElement(element);
        el.src = url;
        el.onload = resolve;
        el.onerror = _ => reject(new Error("Failed to fetch resource: " + url));
        document.body.appendChild(el);
    });
}

/**
 * Fetches a script.
 *
 * @param {string} url
 */
async function fetchScript(url) {
    return fetchResource("script", url);
}

/**
 * Fetches an image.
 *
 * @param {string} url
 */
 async function fetchImage(url) {
    return fetchResource("img", url);
}

/**
 * Returns true when the resource is preloaded via Early Hints.
 *
 * @param {string} url
 * @returns {boolean}
 */
function isPreloadedByEarlyHints(url) {
    const entries = performance.getEntriesByName(url);
    if (entries.length === 0) {
        return false;
    }
    assert_equals(entries.length, 1);
    return entries[0].initiatorType === "early-hints";
}

/**
 * Navigate to the referrer policy test page.
 *
 * @param {string} referrer_policy - A value of Referrer-Policy to test.
 */
function testReferrerPolicy(referrer_policy) {
    const params = new URLSearchParams();
    params.set("referrer-policy", referrer_policy);
    const same_origin_preload_url = SAME_ORIGIN_RESOURCES_URL + "/fetch-and-record-js.h2.py?id=" + token();
    params.set("same-origin-preload-url", same_origin_preload_url);
    const cross_origin_preload_url = CROSS_ORIGIN_RESOURCES_URL + "/fetch-and-record-js.h2.py?id=" + token();
    params.set("cross-origin-preload-url", cross_origin_preload_url);

    const path = "resources/referrer-policy-test-loader.h2.py?" + params.toString();
    const url = new URL(path, window.location);
    window.location.replace(url);
}

/**
 * Navigate to the content security policy basic test. The test page sends an
 * Early Hints response with a cross origin resource preload. CSP headers are
 * configured based on the given policies. A policy should be one of the
 * followings:
 *   "absent" - Do not send Content-Security-Policy header
 *   "allowed" - Set Content-Security-Policy to allow the cross origin preload
 *   "disallowed" - Set Content-Security-Policy to disallow the cross origin  preload
 *
 * @param {string} early_hints_policy - The policy for the Early Hints response
 * @param {string} final_policy - The policy for the final response
 */
function navigateToContentSecurityPolicyBasicTest(
    early_hints_policy, final_policy) {
    const params = new URLSearchParams();
    params.set("resource-origin", CROSS_ORIGIN);
    params.set("resource-url",
        CROSS_ORIGIN_RESOURCES_URL + "/empty.js?" + token());
    params.set("early-hints-policy", early_hints_policy);
    params.set("final-policy", final_policy);

    const url = "resources/csp-basic-loader.h2.py?" + params.toString();
    window.location.replace(new URL(url, window.location));
}

/**
 * Navigate to a test page which sends an Early Hints containing a cross origin
 * preload link with/without Content-Security-Policy header. The CSP header is
 * configured based on the given policy. The test page disallows the preload
 * while the preload is in-flight. The policy should be one of the followings:
 *   "absent" - Do not send Content-Security-Policy header
 *   "allowed" - Set Content-Security-Policy to allow the cross origin preload
 *
 * @param {string} early_hints_policy
 */
function navigateToContentSecurityPolicyDocumentDisallowTest(early_hints_policy) {
    const resource_id = token();
    const params = new URLSearchParams();
    params.set("resource-origin", CROSS_ORIGIN);
    params.set("resource-url",
        CROSS_ORIGIN_RESOURCES_URL + "/delayed-js.h2.py?id=" + resource_id);
    params.set("resume-url",
        CROSS_ORIGIN_RESOURCES_URL + "/resume-delayed-js.h2.py?id=" + resource_id);
    params.set("early-hints-policy", early_hints_policy);

    const url = "resources/csp-document-disallow-loader.h2.py?" + params.toString();
    window.location.replace(new URL(url, window.location));
}

/**
 * Navigate to a test page which sends different Cross-Origin-Embedder-Policy
 * values in an Early Hints response and the final response.
 *
 * @param {string} early_hints_policy - The policy for the Early Hints response
 * @param {string} final_policy - The policy for the final response
 */
function navigateToCrossOriginEmbedderPolicyMismatchTest(
    early_hints_policy, final_policy) {
    const params = new URLSearchParams();
    params.set("resource-url",
        CROSS_ORIGIN_RESOURCES_URL + "/empty-corp-absent.js?" + token());
    params.set("early-hints-policy", early_hints_policy);
    params.set("final-policy", final_policy);

    const url = "resources/coep-mismatch.h2.py?" + params.toString();
    window.location.replace(new URL(url, window.location));
}
