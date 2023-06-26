// Helpers called on the main test HTMLs.
// Functions in `RemoteContext.execute_script()`'s 1st argument are evaluated
// on the executors (`executor.html`), and helpers available on the executors
// are defined in `executor.html`.

const originSameOrigin =
  location.protocol === 'http:' ?
  'http://{{host}}:{{ports[http][0]}}' :
  'https://{{host}}:{{ports[https][0]}}';
const originSameSite =
  location.protocol === 'http:' ?
  'http://{{host}}:{{ports[http][1]}}' :
  'https://{{host}}:{{ports[https][1]}}';
const originCrossSite =
  location.protocol === 'http:' ?
  'http://{{hosts[alt][www]}}:{{ports[http][0]}}' :
  'https://{{hosts[alt][www]}}:{{ports[https][0]}}';

const executorPath =
  '/html/browsers/browsing-the-web/back-forward-cache/resources/executor.html?uuid=';

// Asserts that the executor `target` is (or isn't, respectively)
// restored from BFCache. These should be used in the following fashion:
// 1. Call prepareNavigation() on the executor `target`.
// 2. Navigate the executor to another page.
// 3. Navigate back to the executor `target`.
// 4. Call assert_bfcached() or assert_not_bfcached() on the main test HTML.
async function assert_bfcached(target) {
  const status = await getBFCachedStatus(target);
  assert_implements_optional(status === 'BFCached',
      "Could have been BFCached but actually wasn't");
}

async function assert_not_bfcached(target) {
  const status = await getBFCachedStatus(target);
  assert_implements(status !== 'BFCached',
      'Should not be BFCached but actually was');
}

async function getBFCachedStatus(target) {
  const [loadCount, isPageshowFired, isPageshowPersisted] =
    await target.execute_script(() => [
        window.loadCount, window.isPageshowFired, window.isPageshowPersisted]);

  if (loadCount === 1 && isPageshowFired === true &&
      isPageshowPersisted === true) {
    return 'BFCached';
  } else if (loadCount === 2 && isPageshowFired === false) {
    return 'Not BFCached';
  } else {
    // This can occur for example when this is called before first navigating
    // away (loadCount = 1, isPageshowFired = false), e.g. when
    // 1. sending a script for navigation and then
    // 2. calling getBFCachedStatus() without waiting for the completion of
    //    the script on the `target` page.
    assert_unreached(
      `Got unexpected BFCache status: loadCount = ${loadCount}, ` +
      `isPageshowFired = ${isPageshowFired}, ` +
      `isPageshowPersisted = ${isPageshowPersisted}`);
  }
}

// Always call `await remoteContext.execute_script(waitForPageShow);` after
// triggering to navigation to the page, to wait for pageshow event on the
// remote context.
const waitForPageShow = () => window.pageShowPromise;

// Run a test that navigates A->B->A:
// 1. Page A is opened by `params.openFunc(url)`.
// 2. `params.funcBeforeNavigation(params.argsBeforeNavigation)` is executed
//    on page A.
// 3. The window is navigated to page B on `params.targetOrigin`.
// 4. The window is back navigated to page A (expecting BFCached).
//
// Events `params.events` (an array of strings) are observed on page A and
// `params.expectedEvents` (an array of strings) is expected to be recorded.
// See `event-recorder.js` for event recording.
//
// Parameters can be omitted. See `defaultParams` below for default.
function runEventTest(params, description) {
  const defaultParams = {
    openFunc(url) {
      window.open(
        `${url}&events=${this.events.join(',')}`,
        '_blank',
        'noopener'
      )
    },
    events: ['pagehide', 'pageshow', 'load'],
    expectedEvents: [
      'window.load',
      'window.pageshow',
      'window.pagehide.persisted',
      'window.pageshow.persisted'
    ],
    async funcAfterAssertion(pageA) {
      assert_array_equals(
        await pageA.execute_script(() => getRecordedEvents()),
        this.expectedEvents);
    }
  }
  // Apply defaults.
  params = { ...defaultParams, ...params };

  runBfcacheTest(params, description);
}

async function navigateAndThenBack(pageA, pageB, urlB,
                                   funcBeforeBackNavigation,
                                   argsBeforeBackNavigation) {
  await pageA.execute_script(
    (url) => {
      prepareNavigation(() => {
        location.href = url;
      });
    },
    [urlB]
  );

  await pageB.execute_script(waitForPageShow);
  if (funcBeforeBackNavigation) {
    await pageB.execute_script(funcBeforeBackNavigation,
                               argsBeforeBackNavigation);
  }
  await pageB.execute_script(
    () => {
      prepareNavigation(() => { history.back(); });
    }
  );

  await pageA.execute_script(waitForPageShow);
}

function runBfcacheTest(params, description) {
  const defaultParams = {
    openFunc: url => window.open(url, '_blank', 'noopener'),
    scripts: [],
    funcBeforeNavigation: () => {},
    argsBeforeNavigation: [],
    targetOrigin: originCrossSite,
    funcBeforeBackNavigation: () => {},
    argsBeforeBackNavigation: [],
    shouldBeCached: true,
    funcAfterAssertion: () => {},
  }
  // Apply defaults.
  params = {...defaultParams, ...params };

  promise_test(async t => {
    const pageA = new RemoteContext(token());
    const pageB = new RemoteContext(token());

    const urlA = executorPath + pageA.context_id;
    const urlB = params.targetOrigin + executorPath + pageB.context_id;

    // So that tests can refer to these URLs for assertions if necessary.
    pageA.url = originSameOrigin + urlA;
    pageB.url = urlB;

    params.openFunc(urlA);

    await pageA.execute_script(waitForPageShow);

    for (const src of params.scripts) {
      await pageA.execute_script((src) => {
        const script = document.createElement("script");
        script.src = src;
        document.head.append(script);
        return new Promise(resolve => script.onload = resolve);
      }, [src]);
    }

    await pageA.execute_script(params.funcBeforeNavigation,
                               params.argsBeforeNavigation);
    await navigateAndThenBack(pageA, pageB, urlB,
                              params.funcBeforeBackNavigation,
                              params.argsBeforeBackNavigation);

    if (params.shouldBeCached) {
      await assert_bfcached(pageA);
    } else {
      await assert_not_bfcached(pageA);
    }

    if (params.funcAfterAssertion) {
      await params.funcAfterAssertion(pageA, pageB, t);
    }
  }, description);
}

// Call clients.claim() on the service worker
async function claim(t, worker) {
  const channel = new MessageChannel();
  const saw_message = new Promise(function(resolve) {
    channel.port1.onmessage = t.step_func(function(e) {
      assert_equals(e.data, 'PASS', 'Worker call to claim() should fulfill.');
      resolve();
    });
  });
  worker.postMessage({type: "claim", port: channel.port2}, [channel.port2]);
  await saw_message;
}

// Assigns the current client to a local variable on the service worker.
async function storeClients(t, worker) {
  const channel = new MessageChannel();
  const saw_message = new Promise(function(resolve) {
    channel.port1.onmessage = t.step_func(function(e) {
      assert_equals(e.data, 'PASS', 'storeClients');
      resolve();
    });
  });
  worker.postMessage({type: "storeClients", port: channel.port2}, [channel.port2]);
  await saw_message;
}

// Call storedClients.postMessage("") on the service worker
async function postMessageToStoredClients(t, worker) {
  const channel = new MessageChannel();
  const saw_message = new Promise(function(resolve) {
    channel.port1.onmessage = t.step_func(function(e) {
      assert_equals(e.data, 'PASS', 'postMessageToStoredClients');
      resolve();
    });
  });
  worker.postMessage({type: "postMessageToStoredClients",
                      port: channel.port2}, [channel.port2]);
  await saw_message;
}
