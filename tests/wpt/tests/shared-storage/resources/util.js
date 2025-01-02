// META: script=/common/utils.js
// META: script=/fenced-frame/resources/utils.js
'use strict';

async function IsSharedStorageSelectUrlAllowed() {
  let allowed = true;
  try {
    await sharedStorage.selectURL("operation", [{url: "1.html"}]);
  } catch (e) {
    allowed = false;
  }

  return allowed;
}

// Execute all shared storage methods (excluding createWorklet).
// and capture their errors. Return true if all methods succeed.
async function AreRegularSharedStorageMethodsAllowed() {
  let deniedCount = 0;

  try {
    await window.sharedStorage.worklet.addModule('/shared-storage/resources/simple-module.js');
  } catch (e) {
    ++deniedCount;
  }

  try {
    await window.sharedStorage.run('operation', {keepAlive: true});
  } catch (e) {
    ++deniedCount;
  }

  try {
    await sharedStorage.selectURL("operation", [{url: "1.html"}], {keepAlive: true});
  } catch (e) {
    ++deniedCount;
  }

  try {
    await window.sharedStorage.set('a', 'b');
  } catch (e) {
    ++deniedCount;
  }

  try {
    await window.sharedStorage.append('a', 'b');
  } catch (e) {
    ++deniedCount;
  }

  try {
    await window.sharedStorage.clear();
  } catch (e) {
    ++deniedCount;
  }

  try {
    await window.sharedStorage.delete('a');
  } catch (e) {
    ++deniedCount;
  }

  if (deniedCount === 0)
    return true;

  return false;
}

// Run sharedStorage.worklet.addModule once.
// @param {string} module - The URL to the module.
async function addModuleOnce(module) {
  try {
    await sharedStorage.worklet.addModule(module);
  } catch (e) {
    // Shared Storage needs to have a module added before we can operate on it.
    // It is generated on the fly with this call, and since there's no way to
    // tell through the API if a module already exists, wrap the addModule call
    // in a try/catch so that if it runs a second time in a test, it will
    // gracefully fail rather than bring the whole test down.
  }
}

// Validate the type of the result of sharedStorage.worklet.selectURL.
// @param result - The result of sharedStorage.worklet.selectURL.
// @param {boolean} - Whether sharedStorage.worklet.selectURL is resolved to
//                    a fenced frame config (true) or an urn:uuid (false).
// @return {boolean} Whether sharedStorage.worklet.selectURL returns an expected
//                   result type or not.
function validateSelectURLResult(result, resolve_to_config) {
  if (resolve_to_config) {
    return result instanceof FencedFrameConfig;
  }

  return result.startsWith('urn:uuid:');
}

function updateUrlToUseNewOrigin(url, newOriginString) {
  const origin = url.origin;
  return new URL(url.toString().replace(origin, newOriginString));
}

function appendExpectedKeyAndValue(url, expectedKey, expectedValue) {
  url.searchParams.append('expectedKey', expectedKey);
  url.searchParams.append('expectedValue', expectedValue);
  return url;
}

function parseExpectedKeyAndValueData() {
  const url = new URL(location.href);
  const key = url.searchParams.get('expectedKey');
  const value = url.searchParams.get('expectedValue');
  return {'expectedKey': key, 'expectedValue': value};
}

function appendExpectedKey(url, expectedKey) {
  url.searchParams.append('expectedKey', expectedKey);
  return url;
}

function parseExpectedKeyData() {
  const url = new URL(location.href);
  const key = url.searchParams.get('expectedKey');
  return {'expectedKey': key};
}

async function verifyKeyValueForOrigin(key, value, origin) {
  const outerKey = token();
  const innerKey = token();
  let iframeUrl = generateURL(
      '/shared-storage/resources/verify-key-value.https.html',
      [outerKey, innerKey]);
  iframeUrl = updateUrlToUseNewOrigin(iframeUrl, origin);
  iframeUrl = appendExpectedKeyAndValue(iframeUrl, key, value);

  attachIFrame(iframeUrl);
  const result = await nextValueFromServer(outerKey);
  assert_equals(result, 'verify_key_value_loaded');
}

async function verifyKeyNotFoundForOrigin(key, origin) {
  const outerKey = token();
  const innerKey = token();
  let iframeUrl = generateURL(
      '/shared-storage/resources/verify-key-not-found.https.html',
      [outerKey, innerKey]);
  iframeUrl = updateUrlToUseNewOrigin(iframeUrl, origin);
  iframeUrl = appendExpectedKey(iframeUrl, key);

  attachIFrame(iframeUrl);
  const result = await nextValueFromServer(outerKey);
  assert_equals(result, 'verify_key_not_found_loaded');
}

async function setKeyValueForOrigin(key, value, origin) {
  const outerKey = token();
  let setIframeUrl = generateURL(
      '/shared-storage/resources/set-key-value.https.html', [outerKey]);
  setIframeUrl = updateUrlToUseNewOrigin(setIframeUrl, origin);
  setIframeUrl = appendExpectedKeyAndValue(setIframeUrl, key, value);

  attachIFrame(setIframeUrl);
  const result = await nextValueFromServer(outerKey);
  assert_equals(result, 'set_key_value_loaded');
}

async function deleteKeyForOrigin(key, origin) {
  const outerKey = token();
  let deleteIframeUrl = generateURL(
      '/shared-storage/resources/delete-key.https.html', [outerKey]);
  deleteIframeUrl = updateUrlToUseNewOrigin(deleteIframeUrl, origin);
  deleteIframeUrl = appendExpectedKey(deleteIframeUrl, key);

  attachIFrame(deleteIframeUrl);
  const result = await nextValueFromServer(outerKey);
  assert_equals(result, 'delete_key_loaded');
}

function getFetchedUrls(worker) {
  return new Promise(function(resolve) {
    var channel = new MessageChannel();
    channel.port1.onmessage = function(msg) {
      resolve(msg);
    };
    worker.postMessage({port: channel.port2}, [channel.port2]);
  });
}

function checkInterceptedUrls(worker, expectedRequests) {
  return getFetchedUrls(worker).then(function(msg) {
    let actualRequests = msg.data.requests;
    assert_equals(actualRequests.length, expectedRequests.length);
    assert_equals(
        JSON.stringify(actualRequests), JSON.stringify(expectedRequests));
  });
}

function attachIFrameWithEventListenerForSelectURLStatus(url) {
  const frame = document.createElement('iframe');
  frame.src = url;

  const promise = new Promise((resolve, reject) => {
    window.addEventListener('message', async function handler(evt) {
      if (evt.source === frame.contentWindow && evt.data.selectURLStatus) {
        document.body.removeChild(frame);
        window.removeEventListener('message', handler);
        if (evt.data.selectURLStatus === 'success') {
          resolve(evt.data);
        } else {
          reject(new Error(JSON.stringify(evt.data)));
        }
      }
    });
  });

  document.body.appendChild(frame);
  return promise;
}
