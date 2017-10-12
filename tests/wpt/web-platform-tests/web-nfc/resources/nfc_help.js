'use strict';

const test_text_data = "Test text data.";
const test_text_byte_array = new TextEncoder('utf-8').encode(test_text_data);
const test_number_data = 42;
const test_json_data = {level: 1, score: 100, label: 'Game'};
const test_url_data = "https://w3c.github.io/web-nfc";
const test_buffer_data = new ArrayBuffer(test_text_byte_array.length);

function noop() {};

function createMessage(records) {
  if (records !== undefined) {
    let message = {};
    message.records = records;
    return message;
  }
}

function createRecord(recordType, mediaType, data) {
  let record = {};
  if (recordType !== undefined) {
    record.recordType = recordType;
  }
  if (mediaType !== undefined) {
    record.mediaType = mediaType;
  }
  if (data !== undefined) {
    record.data = data;
  }
  return record;
}

function createTextRecord(text) {
  return createRecord('text', 'text/plain', text);
}

function createJsonRecord(json) {
  return createRecord('json', 'application/json', json);
}

function createOpaqueRecord(buffer) {
  return createRecord('opaque', 'application/octet-stream', buffer);
}

function createUrlRecord(url) {
  return createRecord('url', 'text/plain', url);
}

function assertWebNFCMessagesEqual(a, b) {
  assert_equals(a.records.length, b.records.length);
  for(let i in a.records) {
    let recordA = a.records[i];
    let recordB = b.records[i];
    assert_equals(recordA.recordType, recordB.recordType);
    assert_equals(recordA.mediaType, recordB.mediaType);
    if (recordA.data instanceof ArrayBuffer) {
      assert_array_equals(new Uint8Array(recordA.data),
          new Uint8Array(recordB.data));
    } else if (typeof recordA.data === 'object') {
      assert_object_equals(recordA.data, recordB.data);
    }
    if (typeof recordA.data === 'number'
        || typeof recordA.data === 'string') {
      assert_true(recordA.data == recordB.data);
    }
  }
}

function testNFCMessage(pushedMessage, watchOptions, desc) {
  promise_test(t => {
    return navigator.nfc.push(pushedMessage)
      .then(() => {
        return new Promise(resolve => {
          navigator.nfc.watch((message) => resolve(message), watchOptions);
        }).then((message) => {
          assertWebNFCMessagesEqual(message, pushedMessage);
        });
      });
  }, desc);
}
