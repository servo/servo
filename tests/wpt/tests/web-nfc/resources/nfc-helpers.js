'use strict';
// These tests rely on the User Agent providing an implementation of
// platform nfc backends.
//
// In Chromium-based browsers this implementation is provided by a polyfill
// in order to reduce the amount of test-only code shipped to users. To enable
// these tests the browser must be run with these options:
//
//   --enable-blink-features=MojoJS,MojoJSTest

async function loadChromiumResources() {
  await loadScript('/resources/testdriver.js');
  await loadScript('/resources/testdriver-vendor.js');
  await import('/resources/chromium/nfc-mock.js');
}

async function initialize_nfc_tests() {
  if (typeof WebNFCTest === 'undefined') {
    const script = document.createElement('script');
    script.src = '/resources/test-only-api.js';
    script.async = false;
    const p = new Promise((resolve, reject) => {
      script.onload = () => { resolve(); };
      script.onerror = e => { reject(e); };
    })
    document.head.appendChild(script);
    await p;

    if (isChromiumBased) {
      await loadChromiumResources();
    }
  }
  assert_implements( WebNFCTest, 'WebNFC testing interface is unavailable.');
  let NFCTest = new WebNFCTest();
  await NFCTest.initialize();
  return NFCTest;
}

function nfc_test(func, name, properties) {
  promise_test(async t => {
    let NFCTest = await initialize_nfc_tests();
    t.add_cleanup(async () => {
      await NFCTest.reset();
    });
    await func(t, NFCTest.getMockNFC());
  }, name, properties);
}

const test_text_data = 'Test text data.';
const test_text_byte_array = new TextEncoder().encode(test_text_data);
const test_number_data = 42;
const test_json_data = {level: 1, score: 100, label: 'Game'};
const test_url_data = 'https://w3c.github.io/web-nfc/';
const test_message_origin = 'https://127.0.0.1:8443';
const test_buffer_data = new ArrayBuffer(test_text_byte_array.length);
const test_buffer_view = new Uint8Array(test_buffer_data);
test_buffer_view.set(test_text_byte_array);
const fake_tag_serial_number = 'c0:45:00:02';
const test_record_id = '/test_path/test_id';

const NFCHWStatus = {};
// OS-level NFC setting is ON
NFCHWStatus.ENABLED = 1;
// no NFC chip
NFCHWStatus.NOT_SUPPORTED = NFCHWStatus.ENABLED + 1;
// OS-level NFC setting OFF
NFCHWStatus.DISABLED = NFCHWStatus.NOT_SUPPORTED + 1;

function encodeTextToArrayBuffer(string, encoding) {
  // Only support 'utf-8', 'utf-16', 'utf-16be', and 'utf-16le'.
  assert_true(
      encoding === 'utf-8' || encoding === 'utf-16' ||
      encoding === 'utf-16be' || encoding === 'utf-16le');

  if (encoding === 'utf-8') {
    return new TextEncoder().encode(string).buffer;
  }

  if (encoding === 'utf-16') {
    let uint16array = new Uint16Array(string.length);
    for (let i = 0; i < string.length; i++) {
      uint16array[i] = string.codePointAt(i);
    }
    return uint16array.buffer;
  }

  const littleEndian = encoding === 'utf-16le';
  const buffer = new ArrayBuffer(string.length * 2);
  const view = new DataView(buffer);
  for (let i = 0; i < string.length; i++) {
    view.setUint16(i * 2, string.codePointAt(i), littleEndian);
  }
  return buffer;
}

function createMessage(records) {
  if (records !== undefined) {
    let message = {};
    message.records = records;
    return message;
  }
}

function createRecord(recordType, data, id, mediaType, encoding, lang) {
  let record = {};
  if (recordType !== undefined)
    record.recordType = recordType;
  if (id !== undefined)
    record.id = id;
  if (mediaType !== undefined)
    record.mediaType = mediaType;
  if (encoding !== undefined)
    record.encoding = encoding;
  if (lang !== undefined)
    record.lang = lang;
  if (data !== undefined)
    record.data = data;
  return record;
}

function createTextRecord(data, encoding, lang) {
  return createRecord('text', data, test_record_id, undefined, encoding, lang);
}

function createMimeRecordFromJson(json) {
  return createRecord(
      'mime', new TextEncoder().encode(JSON.stringify(json)),
      test_record_id, 'application/json');
}

function createMimeRecord(buffer) {
  return createRecord(
      'mime', buffer, test_record_id, 'application/octet-stream');
}

function createUnknownRecord(buffer) {
  return createRecord('unknown', buffer, test_record_id);
}

function createUrlRecord(url, isAbsUrl) {
  if (isAbsUrl) {
    return createRecord('absolute-url', url, test_record_id);
  }
  return createRecord('url', url, test_record_id);
}

// Compares NDEFMessageSource that was provided to the API
// (e.g. NDEFReader.write), and NDEFMessage that was received by the
// mock NFC service.
function assertNDEFMessagesEqual(providedMessage, receivedMessage) {
  // If simple data type is passed, e.g. String or ArrayBuffer or
  // ArrayBufferView, convert it to NDEFMessage before comparing.
  // https://w3c.github.io/web-nfc/#dom-ndefmessagesource
  let provided = providedMessage;
  if (providedMessage instanceof ArrayBuffer ||
      ArrayBuffer.isView(providedMessage))
    provided = createMessage([createRecord(
        'mime', providedMessage, undefined /* id */,
        'application/octet-stream')]);
  else if (typeof providedMessage === 'string')
    provided = createMessage([createRecord('text', providedMessage)]);

  assert_equals(provided.records.length, receivedMessage.data.length,
      'NDEFMessages must have same number of NDEFRecords');

  // Compare contents of each individual NDEFRecord
  for (let i = 0; i < provided.records.length; ++i)
    compareNDEFRecords(provided.records[i], receivedMessage.data[i]);
}

// Used to compare two NDEFMessage, one that is received from
// NDEFReader.onreading() EventHandler and another that is provided to mock NFC
// service.
function assertWebNDEFMessagesEqual(message, expectedMessage) {
  assert_equals(message.records.length, expectedMessage.records.length);

  for(let i in message.records) {
    let record = message.records[i];
    let expectedRecord = expectedMessage.records[i];
    assert_equals(record.recordType, expectedRecord.recordType);
    assert_equals(record.mediaType, expectedRecord.mediaType);
    assert_equals(record.id, expectedRecord.id);
    assert_equals(record.encoding, expectedRecord.encoding);
    assert_equals(record.lang, expectedRecord.lang);
    // Compares record data
    assert_array_equals(new Uint8Array(record.data),
          new Uint8Array(expectedRecord.data));
  }
}

function testMultiScanOptions(message, scanOptions, unmatchedScanOptions, desc) {
  nfc_test(async (t, mockNFC) => {
    const ndef1 = new NDEFReader();
    const ndef2 = new NDEFReader();
    const controller = new AbortController();

    // Reading from unmatched ndef will not be triggered
    ndef1.onreading = t.unreached_func("reading event should not be fired.");
    unmatchedScanOptions.signal = controller.signal;
    await ndef1.scan(unmatchedScanOptions);

    const ndefWatcher = new EventWatcher(t, ndef2, ["reading", "readingerror"]);
    const promise = ndefWatcher.wait_for("reading").then(event => {
      controller.abort();
      assertWebNDEFMessagesEqual(event.message, new NDEFMessage(message));
    });
    scanOptions.signal = controller.signal;
    await ndef2.scan(scanOptions);

    mockNFC.setReadingMessage(message);
    await promise;
  }, desc);
}

function testMultiMessages(message, scanOptions, unmatchedMessage, desc) {
  nfc_test(async (t, mockNFC) => {
    const ndef = new NDEFReader();
    const controller = new AbortController();
    const ndefWatcher = new EventWatcher(t, ndef, ["reading", "readingerror"]);
    const promise = ndefWatcher.wait_for("reading").then(event => {
      controller.abort();
      assertWebNDEFMessagesEqual(event.message, new NDEFMessage(message));
    });
    scanOptions.signal = controller.signal;
    await ndef.scan(scanOptions);

    // Unmatched message will not be read
    mockNFC.setReadingMessage(unmatchedMessage);
    mockNFC.setReadingMessage(message);
    await promise;
  }, desc);
}
