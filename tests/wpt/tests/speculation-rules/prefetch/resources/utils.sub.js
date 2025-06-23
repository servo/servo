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
  assert_in_array(requestHeaders.purpose, [undefined, "prefetch"], "The vendor-specific header Purpose, if present, must be 'prefetch'.");
  assert_in_array(requestHeaders['sec-purpose'],
                  ["prefetch", "prefetch;anonymous-client-ip"], description);
}

function assert_prefetched_anonymous_client_ip(requestHeaders, description) {
  assert_in_array(requestHeaders.purpose, [undefined, "prefetch"], "The vendor-specific header Purpose, if present, must be 'prefetch'.");
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
  assert_in_array(requestHeaders.purpose, [undefined, "prefetch"],
      "The vendor-specific header Purpose, if present, must be 'prefetch'.");
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
