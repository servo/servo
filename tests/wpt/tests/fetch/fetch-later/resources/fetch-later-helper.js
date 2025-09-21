/**
 * IMPORTANT: Before using this file, you must also import the following files:
 * - /common/utils.js
 */
'use strict';

const ROOT_NAME = 'fetch/fetch-later';

function parallelPromiseTest(func, description) {
  async_test((t) => {
    Promise.resolve(func(t)).then(() => t.done()).catch(t.step_func((e) => {
      throw e;
    }));
  }, description);
}

/** @enum {string} */
const BeaconDataType = {
  String: 'String',
  ArrayBuffer: 'ArrayBuffer',
  FormData: 'FormData',
  URLSearchParams: 'URLSearchParams',
  Blob: 'Blob',
  File: 'File',
};

/** @enum {string} */
const BeaconDataTypeToSkipCharset = {
  String: '',
  ArrayBuffer: '',
  FormData: '\n\r',  // CRLF characters will be normalized by FormData
  URLSearchParams: ';,/?:@&=+$',  // reserved URI characters
  Blob: '',
  File: '',
};

const BEACON_PAYLOAD_KEY = 'payload';

// Creates beacon data of the given `dataType` from `data`.
// @param {string} data - A string representation of the beacon data. Note that
//     it cannot contain UTF-16 surrogates for all `BeaconDataType` except BLOB.
// @param {BeaconDataType} dataType - must be one of `BeaconDataType`.
// @param {string} contentType - Request Content-Type.
function makeBeaconData(data, dataType, contentType) {
  switch (dataType) {
    case BeaconDataType.String:
      return data;
    case BeaconDataType.ArrayBuffer:
      return new TextEncoder().encode(data).buffer;
    case BeaconDataType.FormData:
      const formData = new FormData();
      if (data.length > 0) {
        formData.append(BEACON_PAYLOAD_KEY, data);
      }
      return formData;
    case BeaconDataType.URLSearchParams:
      if (data.length > 0) {
        return new URLSearchParams(`${BEACON_PAYLOAD_KEY}=${data}`);
      }
      return new URLSearchParams();
    case BeaconDataType.Blob: {
      const options = {type: contentType || undefined};
      return new Blob([data], options);
    }
    case BeaconDataType.File: {
      const options = {type: contentType || 'text/plain'};
      return new File([data], 'file.txt', options);
    }
    default:
      throw Error(`Unsupported beacon dataType: ${dataType}`);
  }
}

// Create a string of `end`-`begin` characters, with characters starting from
// UTF-16 code unit `begin` to `end`-1.
function generateSequentialData(begin, end, skip) {
  const codeUnits = Array(end - begin).fill().map((el, i) => i + begin);
  if (skip) {
    return String.fromCharCode(
        ...codeUnits.filter(c => !skip.includes(String.fromCharCode(c))));
  }
  return String.fromCharCode(...codeUnits);
}

function generatePayload(size) {
  if (size == 0) {
    return '';
  }
  const prefix = String(size) + ':';
  if (size < prefix.length) {
    return Array(size).fill('*').join('');
  }
  if (size == prefix.length) {
    return prefix;
  }

  return prefix + Array(size - prefix.length).fill('*').join('');
}

function generateSetBeaconURL(uuid, options) {
  const host = (options && options.host) || '';
  let url = `${host}/${ROOT_NAME}/resources/set_beacon.py?uuid=${uuid}`;
  if (options) {
    if (options.expectOrigin !== undefined) {
      url = `${url}&expectOrigin=${options.expectOrigin}`;
    }
    if (options.expectPreflight !== undefined) {
      url = `${url}&expectPreflight=${options.expectPreflight}`;
    }
    if (options.expectCredentials !== undefined) {
      url = `${url}&expectCredentials=${options.expectCredentials}`;
    }

    if (options.useRedirectHandler) {
      const redirect = `${host}/common/redirect.py` +
          `?location=${encodeURIComponent(url)}`;
      url = redirect;
    }
  }
  return url;
}

async function poll(asyncFunc, expected) {
  const maxRetries = 30;
  const waitInterval = 100;  // milliseconds.
  const delay = ms => new Promise(res => setTimeout(res, ms));

  let result = {data: []};
  for (let i = 0; i < maxRetries; i++) {
    result = await asyncFunc();
    if (!expected(result)) {
      await delay(waitInterval);
      continue;
    }
    return result;
  }
  return result;
}

// Waits until the `options.count` number of beacon data available from the
// server. Defaults to 1.
// If `options.data` is set, it will be used to compare with the data from the
// response.
async function expectBeacon(uuid, options) {
  const expectedCount =
      (options && options.count !== undefined) ? options.count : 1;

  const res = await poll(
      async () => {
        const res = await fetch(
            `/${ROOT_NAME}/resources/get_beacon.py?uuid=${uuid}`,
            {cache: 'no-store'});
        return await res.json();
      },
      (res) => {
        if (expectedCount == 0) {
          // If expecting no beacon, we should try to wait as long as possible.
          // So always returning false here until `poll()` decides to terminate
          // itself.
          return false;
        }
        return res.data.length == expectedCount;
      });
  if (!options || !options.data) {
    assert_equals(
        res.data.length, expectedCount,
        'Number of sent beacons does not match expected count:');
    return;
  }

  if (expectedCount == 0) {
    assert_equals(
        res.data.length, 0,
        'Number of sent beacons does not match expected count:');
    return;
  }

  const decoder = options && options.percentDecoded ? (s) => {
    // application/x-www-form-urlencoded serializer encodes space as '+'
    // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/encodeURIComponent
    s = s.replace(/\+/g, '%20');
    return decodeURIComponent(s);
  } : (s) => s;

  assert_equals(
      res.data.length, options.data.length,
      `The size of beacon data ${
          res.data.length} from server does not match expected value ${
          options.data.length}.`);
  for (let i = 0; i < options.data.length; i++) {
    assert_equals(
        decoder(res.data[i]), options.data[i],
        'The beacon data does not match expected value.');
  }
}

function generateHTML(script) {
  return `<!DOCTYPE html><body><script>${script}</script></body>`;
}

// Loads `script` into an iframe and appends it to the current document.
// Returns the loaded iframe element.
async function loadScriptAsIframe(script) {
  const iframe = document.createElement('iframe');
  iframe.srcdoc = generateHTML(script);
  const iframeLoaded = new Promise(resolve => iframe.onload = resolve);
  document.body.appendChild(iframe);
  await iframeLoaded;
  return iframe;
}

/**
 * A helper to make a fetchLater request and wait for it being received.
 *
 * This function can also be used when the caller does not care about where a
 * fetchLater() makes request to.
 *
 * @param {!RequestInit} init The request config to pass into fetchLater() call.
 */
async function expectFetchLater(
    init, {targetUrl = undefined, uuid = undefined} = {}) {
  if ((targetUrl && !uuid) || (!targetUrl && uuid)) {
    throw new Error('uuid and targetUrl must be provided together.');
  }
  if (uuid && targetUrl && !targetUrl.includes(uuid)) {
    throw new Error(`Conflicting uuid=${
        uuid} is provided: must also be included in the targetUrl ${
        targetUrl}`);
  }
  if (!uuid) {
    uuid = token();
  }
  if (!targetUrl) {
    targetUrl = generateSetBeaconURL(uuid);
  }

  fetchLater(targetUrl, init);

  await expectBeacon(uuid, {count: 1});
}

/**
 * A helper to append `el` into document and wait for it being loaded.
 * @param {!Element} el
 */
async function loadElement(el) {
  const loaded = new Promise(resolve => el.onload = resolve);
  document.body.appendChild(el);
  await loaded;
}

/**
 * The options to configure a fetchLater() call in an iframe.
 * @record
 */
class FetchLaterIframeOptions {
  constructor() {
    /**
     * @type {string=} The url to pass to the fetchLater() call.
     */
    this.targetUrl;

    /**
     * @type {string=} The uuid to wait for. Must also be part of `targetUrl`.
     */
    this.uuid;

    /**
     * @type {number=} The activateAfter field of DeferredRequestInit to pass
     * to the fetchLater() call.
     * https://whatpr.org/fetch/1647.html#dictdef-deferredrequestinit
     */
    this.activateAfter;

    /**
     * @type {string=} The method field of DeferredRequestInit to pass to the
     * fetchLater() call.
     * https://whatpr.org/fetch/1647.html#dictdef-deferredrequestinit
     */
    this.method;

    /**
     * @type {string=} The referrer field of DeferredRequestInit to pass to the
     * fetchLater() call.
     * https://whatpr.org/fetch/1647.html#requestinit
     */
    this.referrer;

    /**
     * @type {string=} One of the `BeaconDataType` to tell the iframe how to
     * generate the body for its fetchLater() call.
     */
    this.bodyType;

    /**
     * @type {number=} The size to tell the iframe how to generate the body of
     * its fetchLater() call.
     */
    this.bodySize;

    /**
     * @type {bool} Whether to set allow="deferred-fetch" attribute for the
     * iframe. Combing with a Permissions-Policy header, this will enable
     * fetchLater() being used in a cross-origin iframe.
     */
    this.allowDeferredFetch;

    /**
     * @type {string=} The sandbox attribute to apply to the iframe.
     */
    this.sandbox;

    /**
     * @type {FetchLaterIframeExpectation=} The expectation on the iframe's
     * behavior.
     */
    this.expect;
  }
}

/**
 * The enum to classify the messages posted from an iframe that has called
 * fetchLater() API.
 * @enum {string}
 */
const FetchLaterIframeMessageType = {
  // Tells that a fetchLater() call has been executed without any error thrown.
  DONE: 'fetchLater.done',
  // Tells that there are some error thrown from a fetchLater() call.
  ERROR: 'fetchLater.error',
};

/**
 * The enum to indicate what type of iframe behavior the caller is expecting.
 * @enum {number}
 */
const FetchLaterExpectationType = {
  // A fetchLater() call should have been made without any errors.
  DONE: 0,
  // A fetchLater() call is made and an JS error is thrown.
  ERROR_JS: 1,
  // A fetchLater() call is made and an DOMException is thrown.
  ERROR_DOM: 2,
};

class FetchLaterExpectationError extends Error {
  constructor(src, actual, expected) {
    const message = `iframe[src=${src}] threw ${actual}, expected ${expected}`;
    super(message);
  }
}

class FetchLaterIframeExpectation {
  constructor(expectationType, expectedError) {
    this.expectationType = expectationType;
    if (expectationType == FetchLaterExpectationType.DONE && !expectedError) {
      this.expectedErrorType = undefined;
    } else if (
        expectationType == FetchLaterExpectationType.ERROR_JS &&
        typeof expectedError == 'function') {
      this.expectedErrorType = expectedError;
    } else if (
        expectationType == FetchLaterExpectationType.ERROR_DOM &&
        typeof expectedError == 'string') {
      this.expectedDomErrorName = expectedError;
    } else {
      throw Error(`Expectation type "${expectationType}" and expected error "${
          expectedError}" do not match`);
    }
  }

  /**
   * Verifies the message from `e` against the configured expectation.
   *
   * @param {MessageEvent} e
   * @param {string} url The source URL of the iframe where `e` is dispatched
   * from.
   * @return {bool}
   * - Returns true if the expected message event is passed into the function
   *   and the expectation is fulfilled. The caller should be able to safely
   *   remove the message event listener afterwards.
   * - Returns false if the passed in event is not of the expected type. The
   *   caller should continue waiting for another message event and call this
   *   function again.
   * @throws {Error} Throws an error if the expected message event is passed but
   *   the expectation fails. The caller should remove the message event
   *   listener and perform test failure handling.
   */
  run(e, url) {
    if (this.expectationType === FetchLaterExpectationType.DONE) {
      if (e.data.type === FetchLaterIframeMessageType.DONE) {
        return true;
      }
      if (e.data.type === FetchLaterIframeMessageType.ERROR &&
          e.data.error !== undefined) {
        throw new FetchLaterExpectationError(
            url, e.data.error.name, 'no error');
      }
    }

    if (this.expectationType === FetchLaterExpectationType.ERROR_JS) {
      if (e.data.type === FetchLaterIframeMessageType.DONE) {
        throw new FetchLaterExpectationError(
            url, 'nothing', this.expectedErrorType.name);
      }
      if (e.data.type === FetchLaterIframeMessageType.ERROR) {
        if (e.data.error.name === this.expectedErrorType.name) {
          return true;
        }
        throw new FetchLaterExpectationError(
            url, e.data.error, this.expectedErrorType.name);
      }
    }

    if (this.expectationType === FetchLaterExpectationType.ERROR_DOM) {
      if (e.data.type === FetchLaterIframeMessageType.DONE) {
        throw new FetchLaterExpectationError(
            url, 'nothing', this.expectedDomErrorName);
      }
      if (e.data.type === FetchLaterIframeMessageType.ERROR) {
        const actual = e.data.error.name || e.data.error.type;
        if (this.expectedDomErrorName === 'QuotaExceededError') {
          return actual == this.expectedDomErrorName;
        } else if (actual == this.expectedDomErrorName) {
          return true;
        }
        throw new FetchLaterExpectationError(
            url, actual, this.expectedDomErrorName);
      }
    }

    return false;
  }
}

/**
 * A helper to load an iframe of the specified `origin` that makes a fetchLater
 * request to `targetUrl`.
 *
 * If `targetUrl` is not provided, this function generates a target URL by
 * itself.
 *
 * If `expect` is not provided:
 * - If `targetUrl` is not provided, this function will wait for the fetchLater
 *   request being received by the test server before returning.
 * - If `targetUrl` is provided and `uuid` is missing, it will NOT wait for the
 *   request.
 * - If both `targetUrl` and `uuid` are provided, it will wait for the request.
 *
 * Note that the iframe posts various messages back to its parent document.
 *
 * @param {!string} origin The origin URL of the iframe to load.
 * @param {FetchLaterIframeOptions=} nameIgnored
 * @return {!HTMLIFrameElement} the loaded iframe.
 */
async function loadFetchLaterIframe(origin, {
  targetUrl = undefined,
  uuid = undefined,
  activateAfter = undefined,
  referrer = undefined,
  method = undefined,
  bodyType = undefined,
  bodySize = undefined,
  allowDeferredFetch = false,
  sandbox = undefined,
  expect = undefined
} = {}) {
  if (uuid && targetUrl && !targetUrl.includes(uuid)) {
    throw new Error(`Conflicted uuid=${
        uuid} is provided: must also be included in the targetUrl ${
        targetUrl}`);
  }
  if (!uuid) {
    uuid = targetUrl ? undefined : token();
  }
  targetUrl = targetUrl || generateSetBeaconURL(uuid);
  const params = new URLSearchParams(Object.assign(
      {},
      {url: encodeURIComponent(targetUrl)},
      activateAfter !== undefined ? {activateAfter} : null,
      referrer !== undefined ? {referrer} : null,
      method !== undefined ? {method} : null,
      bodyType !== undefined ? {bodyType} : null,
      bodySize !== undefined ? {bodySize} : null,
      ));
  const url =
      `${origin}/fetch/fetch-later/resources/fetch-later.html?${params}`;
  expect =
      expect || new FetchLaterIframeExpectation(FetchLaterExpectationType.DONE);

  const iframe = document.createElement('iframe');
  if (allowDeferredFetch) {
    iframe.allow = 'deferred-fetch';
  }
  if (sandbox) {
    iframe.sandbox = sandbox;
  }
  iframe.src = url;

  const messageReceived = new Promise((resolve, reject) => {
    addEventListener('message', function handler(e) {
      if (e.source !== iframe.contentWindow) {
        return;
      }
      try {
        if (expect.run(e, url)) {
          removeEventListener('message', handler);
          resolve(e.data.type);
        }
      } catch (err) {
        reject(err);
      }
    });
  });

  await loadElement(iframe);
  const messageType = await messageReceived;
  if (messageType === FetchLaterIframeMessageType.DONE && uuid) {
    await expectBeacon(uuid, {count: 1});
  }

  return iframe;
}
