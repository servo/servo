const STORE_URL = '/speculation-rules/prerender/resources/key-value-store.py';

function assertSpeculationRulesIsSupported() {
  assert_implements(
      'supports' in HTMLScriptElement,
      'HTMLScriptElement.supports is not supported');
  assert_implements(
      HTMLScriptElement.supports('speculationrules'),
      '<script type="speculationrules"> is not supported');
}

// Starts prerendering for `url`.
function startPrerendering(url) {
  // Adds <script type="speculationrules"> and specifies a prerender candidate
  // for the given URL.
  // TODO(https://crbug.com/1174978): <script type="speculationrules"> may not
  // start prerendering for some reason (e.g., resource limit). Implement a
  // WebDriver API to force prerendering.
  const script = document.createElement('script');
  script.type = 'speculationrules';
  script.text = `{"prerender": [{"source": "list", "urls": ["${url}"] }] }`;
  document.head.appendChild(script);
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
  while (true) {
    // Fetches the test result from the server.
    const { status, value } = await readValueFromServer(key);
    if (!status) {
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
function loadInitiatorPage() {
  // Used to communicate with the prerendering page.
  const prerenderChannel = new PrerenderChannel('prerender-channel');
  window.addEventListener('unload', () => {
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
  startPrerendering(url.toString());

  // Navigate to the prerendered page after being informed.
  readyToActivate.then(() => {
    window.location = url.toString();
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

async function create_prerendered_page(t, opt = {}) {
  const baseUrl = '/speculation-rules/prerender/resources/exec.py';
  const init_uuid = token();
  const prerender_uuid = token();
  const discard_uuid = token();
  const init_remote = new RemoteContext(init_uuid);
  const prerender_remote = new RemoteContext(prerender_uuid);
  const discard_remote = new RemoteContext(discard_uuid);
  window.open(`${baseUrl}?uuid=${init_uuid}&init`, '_blank', 'noopener');
  const params = new URLSearchParams(baseUrl.search);
  params.set('uuid', prerender_uuid);
  params.set('discard_uuid', discard_uuid);
  for (const p in opt)
    params.set(p, opt[p]);
  const url = `${baseUrl}?${params.toString()}`;

  await init_remote.execute_script(url => {
      const a = document.createElement('a');
      a.href = url;
      a.innerText = 'Activate';
      document.body.appendChild(a);
      const rules = document.createElement('script');
      rules.type = "speculationrules";
      rules.text = JSON.stringify({prerender: [{source: 'list', urls: [url]}]});
      document.head.appendChild(rules);
  }, [url]);

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

    init_remote.execute_script(url => {
        location.href = url;
    }, [url]);
    return Promise.any([prerendering, discarded]);
  }

  async function activate() {
    const prerendering = await tryToActivate();
    if (prerendering !== 'activated')
      throw new Error('Should not be prerendering at this point')
  }

  return {
    exec: (fn, args) => prerender_remote.execute_script(fn, args),
    activate,
    tryToActivate
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
