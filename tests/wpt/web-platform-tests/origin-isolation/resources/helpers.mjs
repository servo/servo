/**
 * Inserts an iframe usable for origin isolation testing, and returns a promise
 * fulfilled when the iframe is loaded and its document.domain is set. The
 * iframe will point to the send-origin-isolation-header.py file, on the
 * designated host
 * @param {string} host - The host used to calculate the iframe's src=""
 * @param {string=} header - The value of the Origin-Isolation header that the
 *   iframe will set. Omit this to set no header.
 * @returns {HTMLIFrameElement} The created iframe element
 */
export async function insertIframe(host, header) {
  const iframe = document.createElement("iframe");
  const navigatePromise = navigateIframe(iframe, host, header);
  document.body.append(iframe);
  await navigatePromise;
  await setBothDocumentDomains(iframe.contentWindow);
  return iframe;
}

/**
 * Navigates an iframe to a page for origin isolation testing, similar to
 * insertIframe but operating on an existing iframe.
 * @param {HTMLIFrameElement} iframeEl - The <iframe> element to navigate
 * @param {string} host - The host to calculate the iframe's new src=""
 * @param {string=} header - The value of the Origin-Isolation header that the
 *   newly-navigated-to page will set. Omit this to set no header.
 * @returns {Promise} a promise fulfilled when the load event fires, or rejected
 *   if the error event fires
 */
export function navigateIframe(iframeEl, host, header) {
  const url = getIframeURL(host, header);

  const waitPromise = waitForIframe(iframeEl, url);
  iframeEl.src = url;
  return waitPromise;
}

/**
 * Returns a promise that is fulfilled when an iframe's load event fires, or
 * rejected when its error event fires.
 * @param {HTMLIFrameElement} iframeEl - The <iframe> element to wait on
 * @param {string} destinationForErrorMessage - A string used in the promise
 *   rejection error message, if the error event fires
 * @returns {Promise} a promise fulfilled when the load event fires, or rejected
 *   if the error event fires
 */
export function waitForIframe(iframeEl, destinationForErrorMessage) {
  return new Promise((resolve, reject) => {
    iframeEl.addEventListener("load", () => resolve());
    iframeEl.addEventListener(
      "error",
      () => reject(new Error(`Could not navigate to ${destinationForErrorMessage}`))
    );
  });
}

/**
 * Expands into a pair of promise_test() calls to ensure that two Windows are in
 * the same agent cluster, by checking both that we can send a
 * WebAssembly.Module, and that we can synchronously access the DOM.
 * @param {Array} testFrames - An array of either the form [self, frameIndex] or
 *   [frameIndex1, frameIndex2], indicating the two Windows under test. E.g.
 *   [self, 0] or [0, 1].
 * @param {string=} testLabelPrefix - A prefix used in the test names. This can
 *   be omitted if testSameAgentCluster is only used once in a test file.
 */
export function testSameAgentCluster(testFrames, testLabelPrefix) {
  const prefix = testLabelPrefix === undefined ? "" : `${testLabelPrefix}: `;

  if (testFrames[0] === self) {
    // Between parent and a child at the index given by testFrames[1]

    promise_test(async () => {
      const frameWindow = frames[testFrames[1]];
      const whatHappened = await sendWasmModule(frameWindow);

      assert_equals(whatHappened, "WebAssembly.Module message received");
    }, `${prefix}message event must occur`);

    promise_test(async () => {
      const frameWindow = frames[testFrames[1]];
      const frameElement = document.querySelectorAll("iframe")[testFrames[1]];

      // Must not throw
      frameWindow.document;

      // Must not throw
      frameWindow.location.href;

      assert_not_equals(frameElement.contentDocument, null, "contentDocument");

      const whatHappened = await accessFrameElement(frameWindow);
      assert_equals(whatHappened, "frameElement accessed successfully");
    }, `${prefix}setting document.domain must give sync access`);
  } else {
    // Between the two children at the index given by testFrames[0] and
    // testFrames[1]

    promise_test(async () => {
      const whatHappened = await sendWasmModuleBetween(testFrames);
      assert_equals(whatHappened, "WebAssembly.Module message received");
    }, `${prefix}message event must occur`);

    promise_test(async () => {
      const whatHappened1 = await accessDocumentBetween(testFrames);
      assert_equals(whatHappened1, "accessed document successfully");

      const whatHappened2 = await accessLocationHrefBetween(testFrames);
      assert_equals(whatHappened2, "accessed location.href successfully");

      // We don't test contentDocument/frameElement for these because accessing
      // those via siblings has to go through the parent anyway.
    }, `${prefix}setting document.domain must give sync access`);
  }
}

/**
 * Expands into a pair of promise_test() calls to ensure that two Windows are in
 * different agent clusters, by checking both that we cannot send a
 * WebAssembly.Module, and that we cannot synchronously access the DOM.
 * @param {Array} testFrames - An array of either the form [self, frameIndex] or
 *   [frameIndex1, frameIndex2], indicating the two Windows under test. E.g.
 *   [self, 0] or [0, 1].
 * @param {string=} testLabelPrefix - A prefix used in the test names. This can
 *   be omitted if testDifferentAgentClusters is only used once in a test file.
 */
export function testDifferentAgentClusters(testFrames, testLabelPrefix) {
  const prefix = testLabelPrefix === undefined ? "" : `${testLabelPrefix}: `;

  if (testFrames[0] === self) {
    // Between parent and a child at the index given by testFrames[1]

    promise_test(async () => {
      const frameWindow = frames[testFrames[1]];
      const whatHappened = await sendWasmModule(frameWindow);

      assert_equals(whatHappened, "messageerror");
    }, `${prefix}messageerror event must occur`);

    promise_test(async () => {
      const frameWindow = frames[testFrames[1]];
      const frameElement = document.querySelectorAll("iframe")[testFrames[1]];

      assert_throws_dom("SecurityError", DOMException, () => {
        frameWindow.document;
      });

      assert_throws_dom("SecurityError", DOMException, () => {
        frameWindow.location.href;
      });

      assert_equals(frameElement.contentDocument, null, "contentDocument");

      const whatHappened = await accessFrameElement(frameWindow);
      assert_equals(whatHappened, "null");
    }, `${prefix}setting document.domain must not give sync access`);
  } else {
    // Between the two children at the index given by testFrames[0] and
    // testFrames[1]

    promise_test(async () => {
      const whatHappened = await sendWasmModuleBetween(testFrames);
      assert_equals(whatHappened, "messageerror");
    }, `${prefix}messageerror event must occur`);

    promise_test(async () => {
      const whatHappened1 = await accessDocumentBetween(testFrames);
      assert_equals(whatHappened1, "SecurityError");

      const whatHappened2 = await accessLocationHrefBetween(testFrames);
      assert_equals(whatHappened2, "SecurityError");

      // We don't test contentDocument/frameElement for these because accessing
      // those via siblings has to go through the parent anyway.
    }, `${prefix}setting document.domain must not give sync access`);
  }
}

/**
 * Creates a promise_test() to check the value of the originIsolationRestricted
 * getter in the given testFrame.
 * @param {Window|number} testFrame - Either self, or a frame index to test.
 * @param {boolean} expected - The expected value for originIsolationRestricted.
 * @param {string=} testLabelPrefix - A prefix used in the test names. This can
 *   be omitted if the function is only used once in a test file.
 */
export function testOriginIsolationRestricted(testFrame, expected, testLabelPrefix) {
  const prefix = testLabelPrefix === undefined ? "" : `${testLabelPrefix}: `;

  if (testFrame === self) {
    // Need to use promise_test() even though it's sync because we use
    // promise_setup() in many tests.
    promise_test(async () => {
      assert_equals(self.originIsolationRestricted, expected);
    }, `${prefix}originIsolationRestricted must equal ${expected}`);
  } else {
    promise_test(async () => {
      const frameWindow = frames[testFrame];
      const result = await getOriginIsolationRestricted(frameWindow);
      assert_equals(result, expected);
    }, `${prefix}originIsolationRestricted must equal ${expected}`);
  }
}

/**
 * Sends a WebAssembly.Module instance to the given Window, and waits for it to
 * send back a message indicating whether it got the module or got a
 * messageerror event. (This relies on the given Window being derived from
 * insertIframe or navigateIframe.)
 * @param {Window} frameWindow - The destination Window
 * @returns {Promise} A promise which will be fulfilled with either
 *   "WebAssembly.Module message received" or "messageerror"
 */
export async function sendWasmModule(frameWindow) {
  // This function is coupled to ./send-origin-isolation-header.py, which ensures
  // that sending such a message will result in a message back.
  frameWindow.postMessage(await createWasmModule(), "*");
  return waitForMessage(frameWindow);
}

/**
 * Sets document.domain (to itself) for both the current Window and the given
 * Window. The latter relies on the given Window being derived from insertIframe
 * or navigateIframe.
 * @param frameWindow - The other Window whose document.domain is to be set
 * @returns {Promise} A promise which will be fulfilled after both
 *   document.domains are set
 */
export async function setBothDocumentDomains(frameWindow) {
  // By setting both this page's document.domain and the iframe's
  // document.domain to the same value, we ensure that they can synchronously
  // access each other, unless they are origin-isolated.
  // NOTE: document.domain being unset is different than it being set to its
  // current value. It is a terrible API.
  document.domain = document.domain;

  // This function is coupled to ./send-origin-isolation-header.py, which ensures
  // that sending such a message will result in a message back.
  frameWindow.postMessage({ command: "set document.domain", newDocumentDomain: document.domain }, "*");
  const whatHappened = await waitForMessage(frameWindow);
  assert_equals(whatHappened, "document.domain is set");
}

async function getOriginIsolationRestricted(frameWindow) {
  // This function is coupled to ./send-origin-isolation-header.py, which ensures
  // that sending such a message will result in a message back.
  frameWindow.postMessage({ command: "get originIsolationRestricted" }, "*");
  return waitForMessage(frameWindow);
}

function getIframeURL(host, header) {
  const url = new URL("send-origin-isolation-header.py", import.meta.url);
  url.host = host;
  if (header !== undefined) {
    url.searchParams.set("header", header);
  }

  return url.href;
}

async function sendWasmModuleBetween(testFrames) {
  const sourceFrame = frames[testFrames[0]];
  const indexIntoParentFrameOfDestination = testFrames[1];

  sourceFrame.postMessage({ command: "send WASM module", indexIntoParentFrameOfDestination }, "*");
  return waitForMessage(sourceFrame);
}

async function accessDocumentBetween(testFrames) {
  const sourceFrame = frames[testFrames[0]];
  const indexIntoParentFrameOfDestination = testFrames[1];

  sourceFrame.postMessage({ command: "access document", indexIntoParentFrameOfDestination }, "*");
  return waitForMessage(sourceFrame);
}

async function accessLocationHrefBetween(testFrames) {
  const sourceFrame = frames[testFrames[0]];
  const indexIntoParentFrameOfDestination = testFrames[1];

  sourceFrame.postMessage({ command: "access location.href", indexIntoParentFrameOfDestination }, "*");
  return waitForMessage(sourceFrame);
}

async function accessFrameElement(frameWindow) {
  frameWindow.postMessage({ command: "access frameElement" }, "*");
  return waitForMessage(frameWindow);
}

function waitForMessage(expectedSource) {
  return new Promise(resolve => {
    const handler = e => {
      if (e.source === expectedSource) {
        resolve(e.data);
        window.removeEventListener("message", handler);
      }
    };
    window.addEventListener("message", handler);
  });
}

// Any WebAssembly.Module will work fine for our tests; we just want to find out
// if it gives message or messageerror. So, we reuse one from the /wasm/ tests.
async function createWasmModule() {
  const response = await fetch("/wasm/serialization/module/resources/incrementer.wasm");
  const ab = await response.arrayBuffer();
  return WebAssembly.compile(ab);
}
