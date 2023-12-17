// META: script=/common/utils.js
// META: script=/fenced-frame/resources/utils.js
'use strict';

async function IsSharedStorageSelectUrlAllowedByPermissionsPolicy() {
  const errorMessage = 'The \"shared-storage-select-url\" Permissions Policy denied the usage of window.sharedStorage.selectURL().';
  let allowedByPermissionsPolicy = true;
  try {
    // Run selectURL() with without addModule() and this should always fail.
    // Check the error message to distinguish between the permissions policy
    // error and the missing addModule() error.
    await sharedStorage.selectURL("operation", [{url: "1.html"}]);
    assert_unreached("did not fail");
  } catch (e) {
    if (e.message === errorMessage) {
      allowedByPermissionsPolicy = false;
    }
  }

  return allowedByPermissionsPolicy;
}

// Execute all shared storage methods and capture their errors. Return true if
// the permissions policy allows all of them; return false if the permissions
// policy disallows all of them. Precondition: only these two outcomes are
// possible.
async function AreSharedStorageMethodsAllowedByPermissionsPolicy() {
  let permissionsPolicyDeniedCount = 0;
  const errorMessage = 'The \"shared-storage\" Permissions Policy denied the method on window.sharedStorage.';

  try {
    await window.sharedStorage.worklet.addModule('/shared-storage/resources/simple-module.js');
  } catch (e) {
    assert_equals(e.message, errorMessage);
    ++permissionsPolicyDeniedCount;
  }

  try {
    await window.sharedStorage.run('operation');
  } catch (e) {
    assert_equals(e.message, errorMessage);
    ++permissionsPolicyDeniedCount;
  }

  try {
    // Run selectURL() with without addModule() and this should always fail.
    // Check the error message to distinguish between the permissions policy
    // error and the missing addModule() error.
    await sharedStorage.selectURL("operation", [{url: "1.html"}]);
    assert_unreached("did not fail");
  } catch (e) {
    if (e.message === errorMessage) {
      ++permissionsPolicyDeniedCount;
    }
  }

  try {
    await window.sharedStorage.set('a', 'b');
  } catch (e) {
    assert_equals(e.message, errorMessage);
    ++permissionsPolicyDeniedCount;
  }

  try {
    await window.sharedStorage.append('a', 'b');
  } catch (e) {
    assert_equals(e.message, errorMessage);
    ++permissionsPolicyDeniedCount;
  }

  try {
    await window.sharedStorage.clear();
  } catch (e) {
    assert_equals(e.message, errorMessage);
    ++permissionsPolicyDeniedCount;
  }

  try {
    await window.sharedStorage.delete('a');
  } catch (e) {
    assert_equals(e.message, errorMessage);
    ++permissionsPolicyDeniedCount;
  }

  if (permissionsPolicyDeniedCount === 0)
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
