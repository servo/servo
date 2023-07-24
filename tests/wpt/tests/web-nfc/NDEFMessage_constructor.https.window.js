// META: script=resources/nfc-helpers.js

'use strict';

test(() => {
  assert_equals(NDEFMessage.length, 1);
  assert_throws_js(TypeError, () => new NDEFMessage());
}, 'NDEFMessage constructor without init dict');

test(() => {
  assert_throws_js(
      TypeError, () => new NDEFMessage(null),
      'NDEFMessageInit#records is a required field.');
}, 'NDEFMessage constructor with null init dict');

test(() => {
  assert_throws_js(
      TypeError, () => new NDEFMessage({dummy_key: 'dummy_value'}),
      'NDEFMessageInit#records is a required field.');
}, 'NDEFMessage constructor without NDEFMessageInit#records field');

test(() => {
  assert_throws_js(
      TypeError, () => new NDEFMessage({records: []}),
      'NDEFMessageInit#records should not be empty.');
}, 'NDEFMessage constructor with NDEFMessageInit#records being empty');

test(() => {
  const message =
      new NDEFMessage(createMessage([createTextRecord(test_text_data)]));
  assert_equals(message.records.length, 1, 'one text record');
  assert_equals(message.records[0].recordType, 'text', 'messageType');
  assert_equals(message.records[0].mediaType, null, 'mediaType');
  assert_equals(message.records[0].encoding, 'utf-8', 'encoding');
  assert_equals(message.records[0].lang, 'en', 'lang');
  assert_true(
      message.records[0].data instanceof DataView, 'data returns a DataView');
  const decoder = new TextDecoder();
  assert_equals(
      decoder.decode(message.records[0].data), test_text_data,
      'data contains the same text content');
}, 'NDEFMessage constructor with a text record');
