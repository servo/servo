'use strict';

const ROOT_NAME = 'pending-beacon';

function parallelPromiseTest(func, description) {
  async_test((t) => {
    Promise.resolve(func(t)).then(() => t.done()).catch(t.step_func((e) => {
      throw e;
    }));
  }, description);
}

const BeaconTypes = [
  {type: PendingPostBeacon, name: 'PendingPostBeacon', expectedMethod: 'POST'},
  {type: PendingGetBeacon, name: 'PendingGetBeacon', expectedMethod: 'GET'},
];

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

function postBeaconSendDataTest(dataType, testData, description, options) {
  parallelPromiseTest(async t => {
    const expectNoData = options && options.expectNoData;
    const expectCount = (options && options.expectCount !== undefined) ?
        options.expectCount :
        1;
    const uuid = token();
    const url =
        generateSetBeaconURL(uuid, (options && options.urlOptions) || {});
    const beacon = new PendingPostBeacon(url);
    assert_equals(beacon.method, 'POST', 'must be POST to call setData().');

    if (options && options.setCookie) {
      document.cookie = options.setCookie;
    }

    beacon.setData(makeBeaconData(
        testData, dataType, (options && options.contentType) || {}));
    beacon.sendNow();

    const expectedData = expectNoData ? null : testData;
    const percentDecoded =
        !expectNoData && dataType === BeaconDataType.URLSearchParams;
    await expectBeacon(uuid, {
      count: expectCount,
      data: [expectedData],
      percentDecoded: percentDecoded
    });
  }, `PendingPostBeacon(${dataType}): ${description}`);
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
