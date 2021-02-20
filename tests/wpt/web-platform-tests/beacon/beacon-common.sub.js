'use strict';

const EMPTY = 'empty';
const SMALL = 'small';
const LARGE = 'large';
const MAX = 'max';
const TOOLARGE = 'toolarge';

const STRING = 'string';
const ARRAYBUFFER = 'arraybuffer';
const FORM = 'form';
const BLOB = 'blob';

function getContentType(type) {
  switch (type) {
    case STRING:
      return 'text/plain;charset=UTF-8';
    case ARRAYBUFFER:
      return null;
    case FORM:
      return 'multipart/form-data';
    case BLOB:
      return null;
    default:
      throw Error(`invalid type: ${type}`);
  }
}

// Create a payload with the given size and type.
// `sizeString` must be one of EMPTY, SMALL, LARGE, MAX, TOOLARGE.
// `type` must be one of STRING, ARRAYBUFFER, FORM, BLOB.
// `contentType` is effective only if `type` is BLOB.
function makePayload(sizeString, type, contentType) {
  let size = 0;
  switch (sizeString) {
    case EMPTY:
      size = 0;
      break;
    case SMALL:
      size = 10;
      break;
    case LARGE:
      size = 10 * 1000;
      break;
    case MAX:
      if (type === FORM) {
        throw Error('Not supported');
      }
      size = 65536;
      break;
    case TOOLARGE:
      size = 65537;
      break;
    default:
      throw Error('invalid size');
  }

  let data = '';
  if (size > 0) {
    const prefix = String(size) + ':';
    data = prefix + Array(size - prefix.length).fill('*').join('');
  }

  switch (type) {
    case STRING:
      return data;
    case ARRAYBUFFER:
      return new TextEncoder().encode(data).buffer;
    case FORM:
      const formData = new FormData();
      if (size > 0) {
        formData.append('payload', data);
      }
      return formData;
    case BLOB:
      const options = contentType ? {type: contentType} : undefined;
      const blob = new Blob([data], options);
      return blob;
    default:
      throw Error('invalid type');
  }
}

function parallelPromiseTest(func, description) {
  async_test((t) => {
    Promise.resolve(func(t)).then(() => t.done()).catch(t.step_func((e) => {
      throw e;
    }));
  }, description);
}

// Poll the server for the test result.
async function waitForResult(id, expectedError = null) {
  const url = `/beacon/resources/beacon.py?cmd=stat&id=${id}`;
  for (let i = 0; i < 30; ++i) {
    const response = await fetch(url);
    const text = await response.text();
    const results = JSON.parse(text);

    if (results.length === 0) {
      await new Promise(resolve => step_timeout(resolve, 100));
      continue;
    }
    assert_equals(results.length, 1, `bad response: '${text}'`);
    const result = results[0];
    // null JSON values parse as null, not undefined
    assert_equals(result.error, expectedError, 'error recorded in stash');
    return result;
  }
  assert_true(false, 'timeout');
}
