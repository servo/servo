/**
 * Utilities for initiating prefetch via speculation rules.
 */

// Resolved URL to find this script.
const SR_PREFETCH_UTILS_URL = new URL(document.currentScript.src, document.baseURI);

// If (and only if) you are writing a test that depends on
// `requires: ["anonymous-client-ip-when-cross-origin"]`, then you must use this
// host as the cross-origin host. (If you need a generic cross-origin host, use
// `get_host_info().NOTSAMESITE_HOST` or similar instead.)
//
// TODO(domenic): document in the web platform tests server infrastructure that
// such a host must exist, and possibly separate it from `{{hosts[alt][]}}`.
const CROSS_ORIGIN_HOST_THAT_WORKS_WITH_ACIWCO = "{{hosts[alt][]}}";

class PrefetchAgent extends RemoteContext {
  constructor(uuid, t) {
    super(uuid);
    this.t = t;
  }

  getExecutorURL(options = {}) {
    let {hostname, username, password, protocol, executor, ...extra} = options;
    let params = new URLSearchParams({uuid: this.context_id, ...extra});
    if(executor === undefined) {
      executor = "executor.sub.html";
    }
    let url = new URL(`${executor}?${params}`, SR_PREFETCH_UTILS_URL);
    if(hostname !== undefined) {
      url.hostname = hostname;
    }
    if(username !== undefined) {
      url.username = username;
    }
    if(password !== undefined) {
      url.password = password;
    }
    if(protocol !== undefined) {
      url.protocol = protocol;
      url.port = protocol === "https" ? "{{ports[https][0]}}" : "{{ports[http][0]}}";
    }
    return url;
  }

  // Requests prefetch via speculation rules.
  //
  // In the future, this should also use browser hooks to force the prefetch to
  // occur despite heuristic matching, etc., and await the completion of the
  // prefetch.
  async forceSinglePrefetch(url, extra = {}, wait_for_completion = true) {
    return this.forceSpeculationRules(
      {
        prefetch: [{source: 'list', urls: [url], ...extra}]
      }, wait_for_completion);
  }

  async forceSpeculationRules(rules, wait_for_completion = true) {
    await this.execute_script((rules) => {
      insertSpeculationRules(rules);
    }, [rules]);
    if (!wait_for_completion) {
      return Promise.resolve();
    }
    return new Promise(resolve => this.t.step_timeout(resolve, 2000));
  }

  // Hook-based counterpart of `forceSinglePrefetch()`. Instead of waiting a
  // fixed 2s (which is unreliable), this awaits the prefetch for `url` reaching
  // `targetStatus` (default 'ready') via the
  // `speculation.prefetchStatusUpdated` WebDriver BiDi test hook. See
  // `waitForPrefetchStatus()` for requirements.
  async forceSinglePrefetchAndWaitForStatus(
      url, extra = {}, targetStatus = PRELOADING_STATUS.READY) {
    return this.forceSpeculationRulesAndWaitForStatus(
        {prefetch: [{source: 'list', urls: [url], ...extra}]}, url,
        targetStatus);
  }

  // Hook-based counterpart of `forceSpeculationRules()`. Inserts `rules` in
  // this (remote) context and resolves once the prefetch for `awaitedUrl`
  // reaches `targetStatus` (default 'ready'). The event listener is registered
  // before the rules are inserted, so early events are not missed.
  //
  // Requires testdriver.js imported with `?feature=bidi`.
  async forceSpeculationRulesAndWaitForStatus(
      rules, awaitedUrl, targetStatus = PRELOADING_STATUS.READY) {
    const unsubscribe = await subscribePrefetchStatusUpdated();
    if (unsubscribe === null) {
      return this.forceSpeculationRules(rules);
    }
    try {
      const statusReached = waitForPrefetchStatus(awaitedUrl, targetStatus);
      await this.execute_script((rules) => {
        insertSpeculationRules(rules);
      }, [rules]);
      return await statusReached;
    } finally {
      await unsubscribe();
    }
  }

  // `url` is the URL to navigate.
  //
  // `expectedDestinationUrl` is the expected URL after navigation.
  // When omitted, `url` is used. When explicitly null, the destination URL is
  // not validated.
  async navigate(url, {expectedDestinationUrl} = {}) {
    await this.execute_script((url) => {
      window.executor.suspend(() => {
        location.href = url;
      });
    }, [url]);
    if (expectedDestinationUrl === undefined) {
      expectedDestinationUrl = url;
    }
    if (expectedDestinationUrl) {
      expectedDestinationUrl.username = '';
      expectedDestinationUrl.password = '';
      assert_equals(
          await this.execute_script(() => location.href),
          expectedDestinationUrl.toString(),
          "expected navigation to reach destination URL");
    }
    await this.execute_script(() => {});
  }

  async getRequestHeaders() {
    return this.execute_script(() => requestHeaders);
  }

  async getResponseCookies() {
    return this.execute_script(() => {
      let cookie = {};
      document.cookie.split(/\s*;\s*/).forEach((kv)=>{
        let [key, value] = kv.split(/\s*=\s*/);
        cookie[key] = value;
      });
      return cookie;
    });
  }

  async getRequestCookies() {
    return this.execute_script(() => window.requestCookies);
  }

  async getRequestCredentials() {
    return this.execute_script(() => window.requestCredentials);
  }

  async setReferrerPolicy(referrerPolicy) {
    return this.execute_script(referrerPolicy => {
      const meta = document.createElement("meta");
      meta.name = "referrer";
      meta.content = referrerPolicy;
      document.head.append(meta);
    }, [referrerPolicy]);
  }

  async getDeliveryType(){
    return this.execute_script(() => {
      return performance.getEntriesByType("navigation")[0].deliveryType;
    });
  }
}

// Produces a URL with a UUID which will record when it's prefetched.
// |extra_params| can be specified to add extra search params to the generated
// URL.
function getPrefetchUrl(extra_params={}) {
  let params = new URLSearchParams({ uuid: token(), ...extra_params });
  return new URL(`prefetch.py?${params}`, SR_PREFETCH_UTILS_URL);
}

// Produces n URLs with unique UUIDs which will record when they are prefetched.
function getPrefetchUrlList(n) {
  return Array.from({ length: n }, () => getPrefetchUrl());
}

async function isUrlPrefetched(url) {
  let response = await fetch(url, {redirect: 'follow'});
  assert_true(response.ok);
  return response.json();
}

// Polls isUrlPrefetched(url) until it reports a prefetch or |deadline_ms| has
// passed, and returns the last count read. Reading the count resets it
// server-side, so callers should assert on the returned value rather than
// calling isUrlPrefetched again.
async function waitUntilPrefetched(t, url, deadline_ms) {
  const start = performance.now();
  let count = 0;
  while (true) {
    count = await isUrlPrefetched(url);
    if (count > 0 || performance.now() - start >= deadline_ms) {
      return count;
    }
    await new Promise(resolve => t.step_timeout(resolve, 100));
  }
}

// Must also include /common/utils.js and /common/dispatcher/dispatcher.js to use this.
async function spawnWindowWithReference(t, options = {}, uuid = token()) {
  let agent = new PrefetchAgent(uuid, t);
  let w = window.open(agent.getExecutorURL(options), '_blank', options);
  t.add_cleanup(() => w.close());
  return {"agent":agent, "window":w};
}

// Must also include /common/utils.js and /common/dispatcher/dispatcher.js to use this.
async function spawnWindow(t, options = {}, uuid = token()) {
  let agent_window_pair = await spawnWindowWithReference(t, options, uuid);
  return agent_window_pair.agent;
}

function insertSpeculationRules(body) {
  let script = document.createElement('script');
  script.type = 'speculationrules';
  script.textContent = JSON.stringify(body);
  document.head.appendChild(script);
}

// -----------------------------------------------------------------------------
// Utilities for awaiting prefetch completion via the WebDriver BiDi
// `speculation.prefetchStatusUpdated` test hook, instead of a fixed timeout.
//
// These are the hook-based counterparts of `forceSpeculationRules()` /
// `forceSinglePrefetch()` (which wait a fixed 2s). Using the test hook makes
// prefetch tests faster and less flaky. The timeout-based helpers remain for
// call sites where no matching event is emitted or where navigation must start
// before the prefetch completes. Hook-based helpers also fall back to the
// timeout when the browser doesn't implement BiDi event subscriptions.
//
// To use these helpers a test must:
//   1. Import testdriver with the bidi feature plus the vendor shim:
//        <script src="/resources/testdriver.js?feature=bidi"></script>
//        <script src="/resources/testdriver-vendor.js"></script>
//   2. For custom rule insertion, subscribe via
//      `subscribePrefetchStatusUpdated()` before calling
//      `waitForPrefetchStatus()`. The force helpers manage their own
//      subscriptions. If subscribing returns null, insert the rules and use the
//      existing timeout instead.
// -----------------------------------------------------------------------------

// `speculation.PreloadingStatus` has known issues:
// https://github.com/WICG/nav-speculation/issues/437
// For now, only use `ready` and `failure` to wait for simple prefetch success
// or failure.
const PRELOADING_STATUS = {
  READY: 'ready',
  FAILURE: 'failure',
};

// Subscribes globally to `speculation.prefetchStatusUpdated` for the entire
// BiDi session (`contexts: null`). This includes spawned windows and cross-site
// or cross-process contexts, where prefetches are typically initiated.
//
// Returns the unsubscribe function from testdriver, or null if the browser
// doesn't implement BiDi event subscriptions. Call the function (e.g. in a
// `t.add_cleanup`) once the events are no longer needed.
async function subscribePrefetchStatusUpdated() {
  try {
    return await test_driver.bidi.speculation.prefetch_status_updated.subscribe(
        {contexts: null});
  } catch (error) {
    if (error === 'error: Action bidi.session.subscribe not implemented') {
      return null;
    }
    throw error;
  }
}

// Returns a Promise that resolves with the matching event once a prefetch for
// `url` reaches `targetStatus` (default 'ready'). The listener is registered
// synchronously, so call this *before* inserting the speculation rules to avoid
// missing early events.
//
// `url` is matched against the event's prefetch URL (the URL from the
// speculation rule, which is stable across redirects). If `url` is null the
// first event reaching `targetStatus` for any URL resolves the promise (useful
// when a test initiates exactly one prefetch).
//
// Unless `targetStatus` is 'failure', an intervening 'failure' event for the
// matched URL rejects the promise so tests fail fast rather than time out.
//
// Requires an active subscription (see `subscribePrefetchStatusUpdated()`) and
// testdriver.js imported with `?feature=bidi`.
function waitForPrefetchStatus(url, targetStatus = PRELOADING_STATUS.READY) {
  const targetHref = url == null ? null : new URL(url, location.href).href;
  return new Promise((resolve, reject) => {
    const removeHandler =
        test_driver.bidi.speculation.prefetch_status_updated.on(event => {
          if (targetHref !== null && event.url !== targetHref) {
            return;
          }
          if (event.status === targetStatus) {
            removeHandler();
            resolve(event);
          } else if (event.status === PRELOADING_STATUS.FAILURE &&
                     targetStatus !== PRELOADING_STATUS.FAILURE) {
            removeHandler();
            reject(new Error(
                `Prefetch for ${event.url} reported 'failure' while waiting ` +
                `for '${targetStatus}'.`));
          }
        });
  });
}

// Creates and appends <a href=|href|> to |insertion point|. If
// |insertion_point| is not specified, document.body is used.
function addLink(href, insertion_point=document.body) {
  const a = document.createElement('a');
  a.href = href;
  insertion_point.appendChild(a);
  return a;
}

// Inserts a prefetch document rule with |predicate|. |predicate| can be
// undefined, in which case the default predicate will be used (i.e. all links
// in document will match).
function insertDocumentRule(predicate, extra_options={}) {
  insertSpeculationRules({
    prefetch: [{
      source: 'document',
      eagerness: 'immediate',
      where: predicate,
      ...extra_options
    }]
  });
}

function assert_prefetched (requestHeaders, description) {
  assert_equals(requestHeaders.purpose, undefined,
    "The vendor-specific header Purpose has been deprecated.");
  assert_in_array(requestHeaders['sec-purpose'],
                  ["prefetch", "prefetch;anonymous-client-ip"], description);
}

function assert_prefetched_anonymous_client_ip(requestHeaders, description) {
  assert_equals(requestHeaders.purpose, undefined,
    "The vendor-specific header Purpose has been deprecated.");
  assert_equals(requestHeaders['sec-purpose'],
                "prefetch;anonymous-client-ip",
                description);
}

function assert_not_prefetched (requestHeaders, description){
  assert_equals(requestHeaders.purpose, undefined, description);
  assert_equals(requestHeaders['sec-purpose'], undefined, description);
}

// If the prefetch request is intercepted and modified by ServiceWorker,
// - "Sec-Purpose: prefetch" header is dropped in Step 33 of
//   https://fetch.spec.whatwg.org/#dom-request
//   because it's a https://fetch.spec.whatwg.org/#forbidden-request-header.
// - "Purpose: prefetch" can still be sent.
// Note that this check passes also for non-prefetch requests, so additional
// checks are needed to distinguish from non-prefetch requests.
function assert_prefetched_without_sec_purpose(requestHeaders, description) {
  assert_equals(requestHeaders.purpose, undefined,
    "The vendor-specific header Purpose has been deprecated.");
  assert_equals(requestHeaders['sec-purpose'], undefined, description);
}

// For ServiceWorker tests.
// `interceptedRequest` is an element of `interceptedRequests` in
// `resources/basic-service-worker.js`.

// The ServiceWorker fetch handler intercepted a prefetching request.
function assert_intercept_prefetch(interceptedRequest, expectedUrl) {
  assert_equals(interceptedRequest.request.url, expectedUrl.toString(),
      "intercepted request URL.");

  assert_prefetched(interceptedRequest.request.headers,
      "Prefetch request should be intercepted.");

  if (new URL(location.href).searchParams.has('clientId')) {
    // https://github.com/WICG/nav-speculation/issues/346
    // https://crbug.com/404294123
    assert_equals(interceptedRequest.resultingClientId, "",
        "resultingClientId shouldn't be exposed.");

    // https://crbug.com/404286918
    // `assert_not_equals()` isn't used for now to create stable failure diffs.
    assert_false(interceptedRequest.clientId === "",
        "clientId should be initiator.");
  }
}

// The ServiceWorker fetch handler intercepted a non-prefetching request.
function assert_intercept_non_prefetch(interceptedRequest, expectedUrl) {
  assert_equals(interceptedRequest.request.url, expectedUrl.toString(),
      "intercepted request URL.");

  assert_not_prefetched(interceptedRequest.request.headers,
      "Non-prefetch request should be intercepted.");

  if (new URL(location.href).searchParams.has('clientId')) {
    // Because this is an ordinal non-prefetch request, `resultingClientId`
    // can be set as normal.
    assert_not_equals(interceptedRequest.resultingClientId, "",
        "resultingClientId can be exposed.");

    assert_not_equals(interceptedRequest.clientId, "",
        "clientId should be initiator.");
  }
}

function assert_served_by_navigation_preload(requestHeaders) {
  assert_equals(
    requestHeaders['service-worker-navigation-preload'],
    'true',
    'Service-Worker-Navigation-Preload');
}

// Use nvs_header query parameter to ask the wpt server
// to populate No-Vary-Search response header.
function addNoVarySearchHeaderUsingQueryParam(url, value){
  if(value){
    url.searchParams.append("nvs_header", value);
  }
}
