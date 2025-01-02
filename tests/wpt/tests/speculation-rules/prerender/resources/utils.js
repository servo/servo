const STORE_URL = '/speculation-rules/prerender/resources/key-value-store.py';

// Starts prerendering for `url`.
//
// `rule_extras` provides additional parameters for the speculation rule used
// to trigger prerendering.
function startPrerendering(url, rule_extras = {}) {
  // Adds <script type="speculationrules"> and specifies a prerender candidate
  // for the given URL.
  // TODO(https://crbug.com/1174978): <script type="speculationrules"> may not
  // start prerendering for some reason (e.g., resource limit). Implement a
  // WebDriver API to force prerendering.
  const script = document.createElement('script');
  script.type = 'speculationrules';
  script.text = JSON.stringify(
      {prerender: [{source: 'list', urls: [url], ...rule_extras}]});
  document.head.appendChild(script);
  return script;
}

class PrerenderChannel extends EventTarget {
  #ids = new Set();
  #url;
  #active = true;

  constructor(name, uid = new URLSearchParams(location.search).get('uid')) {
    super();
    this.#url = `/speculation-rules/prerender/resources/deprecated-broadcast-channel.py?name=${name}&uid=${uid}`;
    (async() => {
      while (this.#active) {
        // Add the "keepalive" option to avoid fetch() results in unhandled
        // rejection with fetch abortion due to window.close().
        // TODO(crbug.com/1356128): After this migration, "keepalive" will not
        // be able to extend the lifetime of a Document, such that it cannot be
        // used here to guarantee the promise resolution.
        const messages = await (await fetch(this.#url, {keepalive: true})).json();
        for (const {data, id} of messages) {
          if (!this.#ids.has(id))
            this.dispatchEvent(new MessageEvent('message', {data}));
          this.#ids.add(id);
        }
      }
    })();
  }

  close() {
    this.#active = false;
  }

  set onmessage(m) {
    this.addEventListener('message', m)
  }

  async postMessage(data) {
    const id = new Date().valueOf();
    this.#ids.add(id);
    // Add the "keepalive" option to prevent messages from being lost due to
    // window.close().
    await fetch(this.#url, {method: 'POST', body: JSON.stringify({data, id}), keepalive: true});
  }
}

// Reads the value specified by `key` from the key-value store on the server.
async function readValueFromServer(key) {
  const serverUrl = `${STORE_URL}?key=${key}`;
  const response = await fetch(serverUrl);
  if (!response.ok)
    throw new Error('An error happened in the server');
  const value = await response.text();

  // The value is not stored in the server.
  if (value === "")
    return { status: false };

  return { status: true, value: value };
}

// Convenience wrapper around the above getter that will wait until a value is
// available on the server.
async function nextValueFromServer(key) {
  let retry = 0;
  while (true) {
    // Fetches the test result from the server.
    let success = true;
    const { status, value } = await readValueFromServer(key).catch(e => {
      if (retry++ >= 5) {
        throw new Error('readValueFromServer failed');
      }
      success = false;
    });
    if (!success || !status) {
      // The test result has not been stored yet. Retry after a while.
      await new Promise(resolve => setTimeout(resolve, 100));
      continue;
    }

    return value;
  }
}

// Writes `value` for `key` in the key-value store on the server.
async function writeValueToServer(key, value) {
  const serverUrl = `${STORE_URL}?key=${key}&value=${value}`;
  await fetch(serverUrl);
}

// Loads the initiator page, and navigates to the prerendered page after it
// receives the 'readyToActivate' message.
//
// `rule_extras` provides additional parameters for the speculation rule used
// to trigger prerendering.
function loadInitiatorPage(rule_extras = {}) {
  // Used to communicate with the prerendering page.
  const prerenderChannel = new PrerenderChannel('prerender-channel');
  window.addEventListener('pagehide', () => {
    prerenderChannel.close();
  });

  // We need to wait for the 'readyToActivate' message before navigation
  // since the prerendering implementation in Chromium can only activate if the
  // response for the prerendering navigation has already been received and the
  // prerendering document was created.
  const readyToActivate = new Promise((resolve, reject) => {
    prerenderChannel.addEventListener('message', e => {
      if (e.data != 'readyToActivate')
        reject(`The initiator page receives an unsupported message: ${e.data}`);
      resolve(e.data);
    });
  });

  const url = new URL(document.URL);
  url.searchParams.append('prerendering', '');
  // Prerender a page that notifies the initiator page of the page's ready to be
  // activated via the 'readyToActivate'.
  startPrerendering(url.toString(), rule_extras);

  // Navigate to the prerendered page after being informed.
  readyToActivate.then(() => {
    if (rule_extras['target_hint'] === '_blank') {
      window.open(url.toString(), '_blank', 'noopener');
    } else {
      window.location = url.toString();
    }
  }).catch(e => {
    const testChannel = new PrerenderChannel('test-channel');
    testChannel.postMessage(
        `Failed to navigate the prerendered page: ${e.toString()}`);
    testChannel.close();
    window.close();
  });
}

// Returns messages received from the given PrerenderChannel
// so that callers do not need to add their own event listeners.
// nextMessage() returns a promise which resolves with the next message.
//
// Usage:
//   const channel = new PrerenderChannel('channel-name');
//   const messageQueue = new BroadcastMessageQueue(channel);
//   const message1 = await messageQueue.nextMessage();
//   const message2 = await messageQueue.nextMessage();
//   message1 and message2 are the messages received.
class BroadcastMessageQueue {
  constructor(c) {
    this.messages = [];
    this.resolveFunctions = [];
    this.channel = c;
    this.channel.addEventListener('message', e => {
      if (this.resolveFunctions.length > 0) {
        const fn = this.resolveFunctions.shift();
        fn(e.data);
      } else {
        this.messages.push(e.data);
      }
    });
  }

  // Returns a promise that resolves with the next message from this queue.
  nextMessage() {
    return new Promise(resolve => {
      if (this.messages.length > 0)
        resolve(this.messages.shift())
      else
        this.resolveFunctions.push(resolve);
    });
  }
}

// Returns <iframe> element upon load.
function createFrame(url) {
  return new Promise(resolve => {
      const frame = document.createElement('iframe');
      frame.src = url;
      frame.onload = () => resolve(frame);
      document.body.appendChild(frame);
    });
}

/**
 * Creates a prerendered page.
 * @param {Object} params - Additional query params for navigations.
 * @param {URLSearchParams} [params.initiator] - For the page that triggers
 *     prerendering.
 * @param {URLSearchParams} [params.prerendering] - For prerendering navigation.
 * @param {URLSearchParams} [params.activating] - For activating navigation.
 * @param {Object} opt - Controls creation of prerendered pages.
 * @param {boolean} [opt.prefetch] - When this is true, prefetch is also
 *     triggered before prerendering.
 * @param {Object} rule_extras - Additional params for the speculation rule used
 *     to trigger prerendering.
 */
async function create_prerendered_page(t, params = {}, opt = {}, rule_extras = {}) {
  const baseUrl = '/speculation-rules/prerender/resources/exec.py';
  const init_uuid = token();
  const prerender_uuid = token();
  const discard_uuid = token();
  const init_remote = new RemoteContext(init_uuid);
  const prerender_remote = new RemoteContext(prerender_uuid);
  const discard_remote = new RemoteContext(discard_uuid);

  const init_params = new URLSearchParams();
  init_params.set('uuid', init_uuid);
  if ('initiator' in params) {
    for (const [key, value] of params.initiator.entries()) {
      init_params.set(key, value);
    }
  }
  window.open(`${baseUrl}?${init_params.toString()}&init`, '_blank', 'noopener');

  // Construct a URL for prerendering.
  const prerendering_params = new URLSearchParams();
  prerendering_params.set('uuid', prerender_uuid);
  prerendering_params.set('discard_uuid', discard_uuid);
  if ('prerendering' in params) {
    for (const [key, value] of params.prerendering.entries()) {
      prerendering_params.set(key, value);
    }
  }
  const prerendering_url = `${baseUrl}?${prerendering_params.toString()}`;

  // Construct a URL for activation. If `params.activating` is provided, the
  // URL is constructed with the params. Otherwise, the URL is the same as
  // `prerendering_url`.
  const activating_url = (() => {
    if ('activating' in params) {
      const activating_params = new URLSearchParams();
      activating_params.set('uuid', prerender_uuid);
      activating_params.set('discard_uuid', discard_uuid);
      for (const [key, value] of params.activating.entries()) {
        activating_params.set(key, value);
      }
      return `${baseUrl}?${activating_params.toString()}`;
    } else {
      return prerendering_url;
    }
  })();

  if (opt.prefetch) {
    await init_remote.execute_script((prerendering_url, rule_extras) => {
        const a = document.createElement('a');
        a.href = prerendering_url;
        a.innerText = 'Activate (prefetch)';
        document.body.appendChild(a);
        const rules = document.createElement('script');
        rules.type = "speculationrules";
        rules.text = JSON.stringify(
            {prefetch: [{source: 'list', urls: [prerendering_url], ...rule_extras}]});
        document.head.appendChild(rules);
    }, [prerendering_url, rule_extras]);

    // Wait for the completion of the prefetch.
    await new Promise(resolve => t.step_timeout(resolve, 3000));
  }

  await init_remote.execute_script((prerendering_url, rule_extras) => {
      const a = document.createElement('a');
      a.href = prerendering_url;
      a.innerText = 'Activate';
      document.body.appendChild(a);
      const rules = document.createElement('script');
      rules.type = "speculationrules";
      rules.text = JSON.stringify({prerender: [{source: 'list', urls: [prerendering_url], ...rule_extras}]});
      document.head.appendChild(rules);
  }, [prerendering_url, rule_extras]);

  await Promise.any([
    prerender_remote.execute_script(() => {
        window.import_script_to_prerendered_page = src => {
            const script = document.createElement('script');
            script.src = src;
            document.head.appendChild(script);
            return new Promise(resolve => script.addEventListener('load', resolve));
        }
    }), new Promise(r => t.step_timeout(r, 3000))
    ]);

  t.add_cleanup(() => {
    init_remote.execute_script(() => window.close());
    discard_remote.execute_script(() => window.close());
    prerender_remote.execute_script(() => window.close());
  });

  async function tryToActivate() {
    const prerendering = prerender_remote.execute_script(() => new Promise(resolve => {
        if (!document.prerendering)
            resolve('activated');
        else document.addEventListener('prerenderingchange', () => resolve('activated'));
    }));

    const discarded = discard_remote.execute_script(() => Promise.resolve('discarded'));

    init_remote.execute_script((activating_url, target_hint) => {
      if (target_hint === '_blank') {
        window.open(activating_url, '_blank', 'noopener');
      } else {
        window.location = activating_url;
      }
    }, [activating_url, rule_extras['target_hint']]);
    return Promise.any([prerendering, discarded]);
  }

  async function activate() {
    const prerendering = await tryToActivate();
    if (prerendering !== 'activated')
      throw new Error('Should not be prerendering at this point')
  }

  // Get the number of network requests for exec.py. This doesn't care about
  // differences in search params.
  async function getNetworkRequestCount() {
    return await (await fetch(prerendering_url + '&get-fetch-count')).text();
  }

  return {
    exec: (fn, args) => prerender_remote.execute_script(fn, args),
    activate,
    tryToActivate,
    getNetworkRequestCount,
    prerenderingURL: (new URL(prerendering_url, document.baseURI)).href,
    activatingURL: (new URL(activating_url, document.baseURI)).href
  };
}


function test_prerender_restricted(fn, expected, label) {
  promise_test(async t => {
    const {exec} = await create_prerendered_page(t);
    let result = null;
    try {
      await exec(fn);
      result = "OK";
    } catch (e) {
      result = e.name;
    }

    assert_equals(result, expected);
  }, label);
}

function test_prerender_defer(fn, label) {
  promise_test(async t => {
    const {exec, activate} = await create_prerendered_page(t);
    let activated = false;
    const deferred = exec(fn);

    const post = new Promise(resolve =>
      deferred.then(result => {
        assert_true(activated, "Deferred operation should occur only after activation");
        resolve(result);
      }));

    await activate();
    activated = true;
    await post;
  }, label);
}

/**
 * Starts prerendering a page from the given referrer `RemoteContextWrapper`,
 * using `<script type="speculationrules">`.
 *
 * See
 * /html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
 * for more details on the `RemoteContextWrapper` framework, and supported fields for extraConfig.
 *
 * The returned `RemoteContextWrapper` for the prerendered remote
 * context will have an extra `url` property, which is used by
 * @see activatePrerenderRC. (Most `RemoteContextWrapper` uses should not care
 * about the URL, but prerendering is unique in that you need to navigate to
 * a prerendered page after creating it.)
 *
 * @param {RemoteContextWrapper} referrerRemoteContext
 * @param {RemoteContextConfig|object} extraConfig
 * @returns {Promise<RemoteContextWrapper>}
 */
function addPrerenderRC(referrerRemoteContext, extraConfig) {
  return referrerRemoteContext.helper.createContext({
    executorCreator(url) {
      return referrerRemoteContext.executeScript(url => {
        const script = document.createElement("script");
        script.type = "speculationrules";
        script.textContent = JSON.stringify({
          prerender: [
            {
              source: "list",
              urls: [url]
            }
          ]
        });
        document.head.append(script);
      }, [url]);
    }, extraConfig
  });
}

/**
 * Activates a prerendered RemoteContextWrapper `prerenderedRC` by navigating
 * the referrer RemoteContextWrapper `referrerRC` to it. If the navigation does
 * not result in a prerender activation, the returned
 * promise will be rejected with a testharness.js AssertionError.
 *
 * See
 * /html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
 * for more on the RemoteContext helper framework.
 *
 * @param {RemoteContextWrapper} referrerRC - The referrer
 *     `RemoteContextWrapper` in which the prerendering was triggered,
 *     probably via `addPrerenderRC()`.
 * @param {RemoteContextWrapper} prerenderedRC - The `RemoteContextWrapper`
 *     pointing to the prerendered content. This is monitored to ensure the
 *     navigation results in a prerendering activation.
 * @param {(string) => Promise<undefined>} [navigateFn] - An optional function
 *     to customize the navigation. It will be passed the URL of the prerendered
 *     content, and will run as a script in `referrerRC` (see
 *     `RemoteContextWrapper.prototype.executeScript`). If not given, navigation
 *     will be done via the `location.href` setter (see
 *     `RemoteContextWrapper.prototype.navigateTo`).
 * @returns {Promise<undefined>}
 */
async function activatePrerenderRC(referrerRC, prerenderedRC, navigateFn) {
  // Store a promise that will fulfill when the prerenderingchange event fires.
  await prerenderedRC.executeScript(() => {
    window.activatedPromise = new Promise(resolve => {
      document.addEventListener("prerenderingchange", () => resolve("activated"));
    });
  });

  if (navigateFn === undefined) {
    referrerRC.navigateTo(prerenderedRC.url);
  } else {
    referrerRC.navigate(navigateFn, [prerenderedRC.url]);
  }

  // Wait until that event fires. If the activation fails and a normal
  // navigation happens instead, then prerenderedRC will start pointing to that
  // other page, where window.activatedPromise is undefined. In that case this
  // assert will fail since undefined !== "activated".
  assert_equals(
    await prerenderedRC.executeScript(() => window.activatedPromise),
    "activated",
    "The prerendered page must be activated; instead a normal navigation happened."
  );
}

async function getActivationStart(prerenderedRC) {
  return await prerenderedRC.executeScript(() => {
    const entry = performance.getEntriesByType("navigation")[0];
    return entry.activationStart;
  });;
}

// Used by the opened window, to tell the main test runner to terminate a
// failed test.
function failTest(reason, uid) {
  const bc = new PrerenderChannel('test-channel', uid);
  bc.postMessage({result: 'FAILED', reason});
  bc.close();
}

// Retrieves a target hint from URLSearchParams of the current window and
// returns it. Throw an Error if it doesn't have the valid target hint param.
function getTargetHint() {
  const params = new URLSearchParams(window.location.search);
  const target_hint = params.get('target_hint');
  if (target_hint === null)
    throw new Error('window.location does not have a target hint param');
  if (target_hint !== '_self' && target_hint !== '_blank')
    throw new Error('window.location does not have a valid target hint param');
  return target_hint;
}
