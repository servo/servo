// META: script=resources/nfc-helpers.js

'use strict';

test(() => {
  assert_equals(NDEFReadingEvent.length, 2);
  assert_throws_js(TypeError, () => new NDEFReadingEvent('message'));
}, 'NDEFReadingEvent constructor without init dict');

test(() => {
  assert_throws_js(
      TypeError,
      () => new NDEFReadingEvent('type', {serialNumber: '', message: null}),
      'NDEFMessageInit#records is a required field.');
}, 'NDEFReadingEvent constructor failed to construct its NDEFMessage');

test(() => {
  const message = createMessage([createMimeRecordFromJson(test_buffer_data)]);
  const event =
      new NDEFReadingEvent('type', {serialNumber: null, message: message});
  assert_equals(event.serialNumber, '', 'serialNumber');
}, 'NDEFReadingEvent constructor with null serialNumber');

test(() => {
  const message = createMessage([createMimeRecordFromJson(test_buffer_data)]);
  const event = new NDEFReadingEvent('type', {message: message});
  assert_equals(event.serialNumber, '', 'serialNumber');
}, 'NDEFReadingEvent constructor with serialNumber not present');

test(() => {
  const message = createMessage([createMimeRecord(test_buffer_data)]);
  const event =
      new NDEFReadingEvent('type', {serialNumber: '', message: message});
  assert_equals(event.type, 'type', 'type');
  assert_equals(event.serialNumber, '', 'serialNumber');
  assertWebNDEFMessagesEqual(event.message, new NDEFMessage(message));
}, 'NDEFReadingEvent constructor with valid parameters');

test(() => {
  const record_init = createTextRecord(test_text_data);
  const event = new NDEFReadingEvent(
      'type', {serialNumber: '', message: createMessage([record_init])});
  assert_equals(event.type, 'type', 'type');
  assert_equals(event.serialNumber, '', 'serialNumber');
  assert_equals(1, event.message.records.length, 'only 1 record');

  const record = new NDEFRecord(record_init);
  assert_equals(record.recordType, 'text', 'recordType');
  assert_equals(record.mediaType, null, 'mediaType');
  assert_equals(record.encoding, 'utf-8', 'encoding');
  assert_equals(record.lang, 'en', 'lang');

  assert_equals(event.message.records[0].recordType, 'text', 'recordType');
  assert_equals(event.message.records[0].mediaType, null, 'mediaType');
  assert_equals(event.message.records[0].encoding, 'utf-8', 'encoding');
  assert_equals(event.message.records[0].lang, 'en', 'lang');
}, 'NDEFReadingEvent constructor sets NDEFRecord#lang for the text records it embeds');
