/**
 * Utilities for initiating prefetch via speculation rules.
 */

// Resolved URL to find this script.
const SR_PREFETCH_UTILS_URL = new URL(document.currentScript.src, document.baseURI);
// Hostname for cross origin urls.
const PREFETCH_PROXY_BYPASS_HOST = "{{hosts[alt][]}}";

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
  async forceSinglePrefetch(url, extra = {}) {
    await this.execute_script((url, extra) => {
      insertSpeculationRules({ prefetch: [{source: 'list', urls: [url], ...extra}] });
    }, [url, extra]);
    return new Promise(resolve => this.t.step_timeout(resolve, 2000));
  }

  async navigate(url) {
    await this.execute_script((url) => {
      window.executor.suspend(() => {
        location.href = url;
      });
    }, [url]);
    url.username = '';
    url.password = '';
    assert_equals(
        await this.execute_script(() => location.href),
        url.toString(),
        "expected navigation to reach destination URL");
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

function getRedirectUrl() {
  let params = new URLSearchParams({uuid: token()});
  return new URL(`redirect.py?${params}`, SR_PREFETCH_UTILS_URL);
}

async function isUrlPrefetched(url) {
  let response = await fetch(url, {redirect: 'follow'});
  assert_true(response.ok);
  return response.json();
}

// Must also include /common/utils.js and /common/dispatcher/dispatcher.js to use this.
async function spawnWindow(t, options = {}, uuid = token()) {
  let agent = new PrefetchAgent(uuid, t);
  let w = window.open(agent.getExecutorURL(options), options);
  t.add_cleanup(() => w.close());
  return agent;
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
      where: predicate,
      ...extra_options
    }]
  });
}

function assert_prefetched (requestHeaders, description) {
  assert_in_array(requestHeaders.purpose, ["", "prefetch"], "The vendor-specific header Purpose, if present, must be 'prefetch'.");
  assert_equals(requestHeaders.sec_purpose, "prefetch", description);
}

function assert_not_prefetched (requestHeaders, description){
  assert_equals(requestHeaders.purpose, "", description);
  assert_equals(requestHeaders.sec_purpose, "", description);
}
