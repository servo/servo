// META: script=resources/nfc-helpers.js

// NDEFRecord constructor
// https://w3c.github.io/web-nfc/#dom-ndefrecord

test(() => {
  assert_equals(NDEFRecord.length, 1);
  assert_throws_js(TypeError, () => new NDEFRecord());
}, 'NDEFRecord constructor without init dict');

test(() => {
  assert_throws_js(
      TypeError, () => new NDEFRecord(null),
      'NDEFRecordInit#recordType is a required field.');
}, 'NDEFRecord constructor with null init dict');

test(() => {
  assert_throws_js(
      TypeError,
      () => new NDEFRecord({id: test_record_id, data: test_text_data}),
      'NDEFRecordInit#recordType is a required field.');
}, 'NDEFRecord constructor without NDEFRecordInit#recordType field');

test(() => {
  assert_throws_js(
      TypeError,
      () =>
          new NDEFRecord(createRecord('empty', test_text_data, test_record_id)),
      'id does not apply for empty record type.');
}, 'NDEFRecord constructor with empty record type and id');

test(() => {
  assert_throws_js(
      TypeError,
      () => new NDEFRecord(
          createRecord('empty', test_text_data, test_record_id, 'text/plain')),
      'mediaType does not apply for empty record type.');
  assert_throws_js(
      TypeError,
      () => new NDEFRecord(
          createRecord('text', test_text_data, test_record_id, 'text/plain')),
      'mediaType does not apply for text record type.');
  assert_throws_js(
      TypeError,
      () => new NDEFRecord(
          createRecord('url', test_url_data, test_record_id, 'text/plain')),
      'mediaType does not apply for url record type.');
  assert_throws_js(
      TypeError,
      () => new NDEFRecord(createRecord(
          'absolute-url', test_url_data, test_record_id, 'text/plain')),
      'mediaType does not apply for absolute-url record type.');
  assert_throws_js(
      TypeError,
      () => new NDEFRecord(createRecord(
          'unknown', test_buffer_data, test_record_id,
          'application/octet-stream')),
      'mediaType does not apply for unknown record type.');
  assert_throws_js(
      TypeError,
      () => new NDEFRecord(createRecord(
          'foo.example.com:bar', test_buffer_data, test_record_id,
          'application/octet-stream')),
      'mediaType does not apply for external record type.');
}, 'NDEFRecord constructor should only accept mediaType for mime record type');

test(
    () => {
      {
        const record = new NDEFRecord(createRecord('text', test_text_data));
        assert_equals(record.id, null, 'id');
      } {const record =
             new NDEFRecord(createRecord('text', test_text_data, ''));
         assert_equals(record.id, '', 'id');} {
        const dummy_id = 'https://dummy_host/mypath/myid';
        const record =
            new NDEFRecord(createRecord('text', test_text_data, dummy_id));
        assert_equals(record.id, dummy_id, 'id');
      } {const dummy_id = 'http://dummy_host/mypath/myid';
         const record =
             new NDEFRecord(createRecord('text', test_text_data, dummy_id));
         assert_equals(record.id, dummy_id, 'id');} {
        const dummy_id = 'mypath/myid';
        const record =
            new NDEFRecord(createRecord('text', test_text_data, dummy_id));
        assert_equals(record.id, dummy_id, 'id');
      }
    },
    'NDEFRecord constructor with custom record ids');

test(() => {
  const record = new NDEFRecord(createRecord('empty'));
  assert_equals(record.recordType, 'empty', 'recordType');
  assert_equals(record.mediaType, null, 'mediaType');
  assert_equals(record.id, null, 'id');
  assert_equals(record.encoding, null, 'encoding');
  assert_equals(record.lang, null, 'lang');
  assert_equals(record.data, null, 'data');
  assert_throws_dom(
      'NotSupportedError', () => record.toRecords(),
      'Only smart-poster, external type and local type records could have embedded records.');
}, 'NDEFRecord constructor with empty record type');

test(() => {
  const record = new NDEFRecord(createTextRecord(test_text_data));
  assert_equals(record.recordType, 'text', 'recordType');
  assert_equals(record.mediaType, null, 'mediaType');
  assert_equals(record.id, test_record_id, 'id');
  assert_equals(record.encoding, 'utf-8', 'encoding');
  assert_equals(record.lang, 'en', 'lang');
  const decoder = new TextDecoder();
  assert_equals(
      decoder.decode(record.data), test_text_data,
      'data has the same content with the original dictionary');
  assert_throws_dom(
      'NotSupportedError', () => record.toRecords(),
      'Only smart-poster, external type and local type records could have embedded records.');
}, 'NDEFRecord constructor with text record type and string data');

test(() => {
  const encoder = new TextEncoder();
  const uint8Array = encoder.encode(test_text_data);
  const record = new NDEFRecord(createTextRecord(uint8Array.buffer));
  assert_equals(record.recordType, 'text', 'recordType');
  assert_equals(record.mediaType, null, 'mediaType');
  assert_equals(record.id, test_record_id, 'id');
  // By default, 'utf-8'.
  assert_equals(record.encoding, 'utf-8', 'encoding');
  assert_equals(record.lang, 'en', 'lang');
  const decoder = new TextDecoder();
  assert_equals(
      decoder.decode(record.data), test_text_data,
      'data has the same content with the original dictionary');
  assert_throws_dom(
      'NotSupportedError', () => record.toRecords(),
      'Only smart-poster, external type and local type records could have embedded records.');
}, 'NDEFRecord constructor with text record type and arrayBuffer data');

test(() => {
  const encoder = new TextEncoder();
  const uint8Array = encoder.encode(test_text_data);
  const record = new NDEFRecord(createTextRecord(uint8Array));
  assert_equals(record.recordType, 'text', 'recordType');
  assert_equals(record.mediaType, null, 'mediaType');
  assert_equals(record.id, test_record_id, 'id');
  // By default, 'utf-8'.
  assert_equals(record.encoding, 'utf-8', 'encoding');
  assert_equals(record.lang, 'en', 'lang');
  const decoder = new TextDecoder();
  assert_equals(
      decoder.decode(record.data), test_text_data,
      'data has the same content with the original dictionary');
  assert_throws_dom(
      'NotSupportedError', () => record.toRecords(),
      'Only smart-poster, external type and local type records could have embedded records.');
}, 'NDEFRecord constructor with text record type and arrayBufferView data');

test(() => {
  assert_throws_js(
      TypeError,
      () =>
          new NDEFRecord(createTextRecord(test_text_data, 'random-encoding')));
  assert_throws_js(
      TypeError,
      () => new NDEFRecord(createTextRecord(test_text_data, 'utf-16')));
  // Only 'utf-8' is OK for a DOMString data source.
  const record =
      new NDEFRecord(createTextRecord(test_text_data, 'utf-8', 'fr'));
  assert_equals(record.recordType, 'text', 'recordType');
  assert_equals(record.encoding, 'utf-8', 'encoding');
  assert_equals(record.lang, 'fr', 'lang');
  const decoder = new TextDecoder();
  assert_equals(
      decoder.decode(record.data), test_text_data,
      'data has the same content with the original text');

  assert_throws_js(
      TypeError,
      () => new NDEFRecord(createTextRecord(
          encodeTextToArrayBuffer(test_text_data, 'utf-8'),
          'random-encoding')));
  // The encoding list valid for a BufferSource data source.
  const encodings = ['utf-8', 'utf-16', 'utf-16be', 'utf-16le'];
  for (const encoding of encodings) {
    const record = new NDEFRecord(createTextRecord(
        encodeTextToArrayBuffer(test_text_data, encoding), encoding, 'fr'));
    assert_equals(record.recordType, 'text', 'recordType');
    assert_equals(record.encoding, encoding, 'encoding');
    assert_equals(record.lang, 'fr', 'lang');
    const decoder = new TextDecoder(record.encoding);
    assert_equals(
        decoder.decode(record.data), test_text_data,
        'data has the same content with the original text. encoding: ' +
            encoding);
  }
}, 'NDEFRecord constructor with text record type, encoding, and lang');

test(t => {
  const previous_lang = document.querySelector('html').getAttribute('lang');
  const test_lang = 'fr';
  document.querySelector('html').setAttribute('lang', test_lang);
  t.add_cleanup(() => {
    document.querySelector('html').setAttribute('lang', previous_lang);
  });
  const record = new NDEFRecord(createTextRecord(test_text_data));
  assert_equals(record.recordType, 'text', 'recordType');
  assert_equals(record.mediaType, null, 'mediaType');
  assert_equals(record.id, test_record_id, 'id');
  assert_equals(record.encoding, 'utf-8', 'encoding');
  assert_equals(record.lang, test_lang, 'lang');
  const decoder = new TextDecoder();
  assert_equals(
      decoder.decode(record.data), test_text_data,
      'data has the same content with the original dictionary');
}, 'NDEFRecord constructor with text record type and custom document language');

test(() => {
  const record = new NDEFRecord(createUrlRecord(test_url_data));
  assert_equals(record.recordType, 'url', 'recordType');
  assert_equals(record.mediaType, null, 'mediaType');
  assert_equals(record.id, test_record_id, 'id');
  const decoder = new TextDecoder();
  assert_equals(
      decoder.decode(record.data), test_url_data,
      'data has the same content with the original dictionary');
  assert_throws_dom(
      'NotSupportedError', () => record.toRecords(),
      'Only smart-poster, external type and local type records could have embedded records.');
}, 'NDEFRecord constructor with url record type');

test(() => {
  const record = new NDEFRecord(createUrlRecord(test_url_data, true));
  assert_equals(record.recordType, 'absolute-url', 'recordType');
  assert_equals(record.mediaType, null, 'mediaType');
  assert_equals(record.id, test_record_id, 'id');
  const decoder = new TextDecoder();
  assert_equals(
      decoder.decode(record.data), test_url_data,
      'data has the same content with the original dictionary');
  assert_throws_dom(
      'NotSupportedError', () => record.toRecords(),
      'Only smart-poster, external type and local type records could have embedded records.');
}, 'NDEFRecord constructor with absolute-url record type');

test(() => {
  assert_throws_js(
      TypeError,
      () => new NDEFRecord(createMimeRecord('A string is not a BufferSource')),
      'Only BufferSource is allowed to be the record data.');

  let buffer = new ArrayBuffer(4);
  new Uint8Array(buffer).set([1, 2, 3, 4]);
  // Feed ArrayBuffer.
  {
    const record = new NDEFRecord(createMimeRecord(buffer));
    assert_equals(record.recordType, 'mime', 'recordType');
    assert_equals(record.mediaType, 'application/octet-stream', 'mediaType');
    assert_equals(record.id, test_record_id, 'id');
    assert_array_equals(
        new Uint8Array(record.data.buffer), [1, 2, 3, 4],
        'data has the same content with the original buffer');
    assert_throws_dom(
        'NotSupportedError', () => record.toRecords(),
        'Only smart-poster, external type and local type records could have embedded records.');
  }
  // Feed ArrayBufferView.
  {
    let buffer_view = new Uint8Array(buffer, 1);
    const record = new NDEFRecord(createMimeRecord(buffer_view));
    assert_equals(record.recordType, 'mime', 'recordType');
    assert_equals(record.id, test_record_id, 'id');
    assert_array_equals(
        new Uint8Array(record.data.buffer), [2, 3, 4],
        'data has the same content with the original buffer view');
    assert_throws_dom(
        'NotSupportedError', () => record.toRecords(),
        'Only smart-poster, external type and local type records could have embedded records.');
  }
}, 'NDEFRecord constructor with mime record type and stream data');

test(() => {
  const record = new NDEFRecord(createMimeRecordFromJson(test_json_data));
  assert_equals(record.recordType, 'mime', 'recordType');
  assert_equals(record.mediaType, 'application/json', 'mediaType');
  assert_equals(record.id, test_record_id, 'id');
  assert_object_equals(
      JSON.parse(new TextDecoder().decode(record.data)), test_json_data,
      'data has the same content with the original json');
  assert_throws_dom(
      'NotSupportedError', () => record.toRecords(),
      'Only smart-poster, external type and local type records could have embedded records.');
}, 'NDEFRecord constructor with mime record type and json data');

test(() => {
  assert_throws_js(
      TypeError,
      () =>
          new NDEFRecord(createUnknownRecord('A string is not a BufferSource')),
      'Only BufferSource is allowed to be the record data.');

  let buffer = new ArrayBuffer(4);
  new Uint8Array(buffer).set([1, 2, 3, 4]);
  // Feed ArrayBuffer.
  {
    const record = new NDEFRecord(createUnknownRecord(buffer));
    assert_equals(record.recordType, 'unknown', 'recordType');
    assert_equals(record.id, test_record_id, 'id');
    assert_array_equals(
        new Uint8Array(record.data.buffer), [1, 2, 3, 4],
        'data has the same content with the original buffer');
    assert_throws_dom(
        'NotSupportedError', () => record.toRecords(),
        'Only smart-poster, external type and local type records could have embedded records.');
  }
  // Feed ArrayBufferView.
  {
    let buffer_view = new Uint8Array(buffer, 1);
    const record = new NDEFRecord(createUnknownRecord(buffer_view));
    assert_equals(record.recordType, 'unknown', 'recordType');
    assert_equals(record.id, test_record_id, 'id');
    assert_array_equals(
        new Uint8Array(record.data.buffer), [2, 3, 4],
        'data has the same content with the original buffer view');
    assert_throws_dom(
        'NotSupportedError', () => record.toRecords(),
        'Only smart-poster, external type and local type records could have embedded records.');
  }
}, 'NDEFRecord constructor with unknown record type');

test(() => {
  assert_throws_js(
      TypeError,
      () => new NDEFRecord(createRecord(
          'foo.eXamPle.com:bAr*-',
          'A string is not a BufferSource or NDEFMessageInit')),
      'Only BufferSource and NDEFMessageInit are allowed to be the record data.');

  let buffer = new ArrayBuffer(4);
  new Uint8Array(buffer).set([1, 2, 3, 4]);
  // Feed ArrayBuffer.
  {
    const record = new NDEFRecord(
        createRecord('foo.eXamPle.com:bAr*-', buffer, test_record_id));
    assert_equals(record.recordType, 'foo.eXamPle.com:bAr*-', 'recordType');
    assert_equals(record.mediaType, null, 'mediaType');
    assert_equals(record.id, test_record_id, 'id');
    assert_array_equals(
        new Uint8Array(record.data.buffer), [1, 2, 3, 4],
        'data has the same content with the original buffer');
    assert_equals(
        record.toRecords(), null,
        'toRecords() returns null if the payload is not an NDEF message.');
  }
  // Feed ArrayBufferView.
  {
    let buffer_view = new Uint8Array(buffer, 1);
    const record = new NDEFRecord(
        createRecord('foo.eXamPle.com:bAr*-', buffer_view, test_record_id));
    assert_equals(record.recordType, 'foo.eXamPle.com:bAr*-', 'recordType');
    assert_equals(record.mediaType, null, 'mediaType');
    assert_equals(record.id, test_record_id, 'id');
    assert_array_equals(
        new Uint8Array(record.data.buffer), [2, 3, 4],
        'data has the same content with the original buffer view');
    assert_equals(
        record.toRecords(), null,
        'toRecords() returns null if the payload is not an NDEF message.');
  }
  // Feed NDEFMessageInit.
  {
    const payload_message = createMessage([createTextRecord(test_text_data)]);
    const record = new NDEFRecord(createRecord(
        'foo.eXamPle.com:bAr*-', payload_message, 'dummy_record_id'));
    assert_equals(record.recordType, 'foo.eXamPle.com:bAr*-', 'recordType');
    assert_equals(record.mediaType, null, 'mediaType');
    assert_equals(record.id, 'dummy_record_id', 'id');
    const embedded_records = record.toRecords();
    assert_equals(embedded_records.length, 1, 'Only one embedded record.');
    // The only one embedded record has correct content.
    assert_equals(embedded_records[0].recordType, 'text', 'recordType');
    assert_equals(embedded_records[0].mediaType, null, 'mediaType');
    assert_equals(embedded_records[0].id, test_record_id, 'id');
    const decoder = new TextDecoder();
    assert_equals(
        decoder.decode(embedded_records[0].data), test_text_data,
        'data has the same content with the original dictionary');
  }
}, 'NDEFRecord constructor with external record type');

test(() => {
  assert_throws_js(
      TypeError, () => new NDEFRecord(createRecord(':xyz', test_buffer_data)),
      'The local type record must be embedded in the payload of another record (smart-poster, external, or local)');

  // The following test cases use an external type record embedding our target
  // local type record.

  const local_record =
      createRecord(':xyz', undefined /* data */, 'dummy_id_for_local_type');
  const payload_message = createMessage([local_record]);
  const external_record_embedding_local_record =
      createRecord('example.com:foo', payload_message);

  local_record.data = 'A string is not a BufferSource or NDEFMessageInit';
  assert_throws_js(
      TypeError, () => new NDEFRecord(external_record_embedding_local_record),
      'Only BufferSource and NDEFMessageInit are allowed to be the record data.');

  let buffer = new ArrayBuffer(4);
  new Uint8Array(buffer).set([1, 2, 3, 4]);
  // Feed ArrayBuffer.
  {
    local_record.data = buffer;
    const record = new NDEFRecord(external_record_embedding_local_record);
    const embedded_records = record.toRecords();
    assert_equals(
        embedded_records.length, 1, 'Only the one embedded local record.');
    // The embedded local record is actually from |local_record|.
    assert_equals(embedded_records[0].recordType, ':xyz', 'recordType');
    assert_equals(embedded_records[0].mediaType, null, 'mediaType');
    assert_equals(embedded_records[0].id, 'dummy_id_for_local_type', 'id');
    assert_array_equals(
        new Uint8Array(embedded_records[0].data.buffer), [1, 2, 3, 4],
        'data has the same content with the original buffer');
    assert_equals(
        embedded_records[0].toRecords(), null,
        'toRecords() returns null if the payload is not an NDEF message.');
  }
  // Feed ArrayBufferView.
  {
    let buffer_view = new Uint8Array(buffer, 1);
    local_record.data = buffer_view;
    const record = new NDEFRecord(external_record_embedding_local_record);
    const embedded_records = record.toRecords();
    assert_equals(
        embedded_records.length, 1, 'Only the one embedded local record.');
    // The embedded local record is actually from |local_record|.
    assert_equals(embedded_records[0].recordType, ':xyz', 'recordType');
    assert_equals(embedded_records[0].mediaType, null, 'mediaType');
    assert_equals(embedded_records[0].id, 'dummy_id_for_local_type', 'id');
    assert_array_equals(
        new Uint8Array(embedded_records[0].data.buffer), [2, 3, 4],
        'data has the same content with the original buffer view');
    assert_equals(
        embedded_records[0].toRecords(), null,
        'toRecords() returns null if the payload is not an NDEF message.');
  }
  // Feed NDEFMessageInit.
  {
    const payload_message = createMessage([createTextRecord(test_text_data)]);
    local_record.data = payload_message;
    const record = new NDEFRecord(external_record_embedding_local_record);
    const embedded_records = record.toRecords();
    assert_equals(
        embedded_records.length, 1, 'Only the one embedded local record.');
    // The embedded local record is actually from |local_record|.
    assert_equals(embedded_records[0].recordType, ':xyz', 'recordType');
    assert_equals(embedded_records[0].mediaType, null, 'mediaType');
    assert_equals(embedded_records[0].id, 'dummy_id_for_local_type', 'id');
    // The embedded local record embeds another text record that's from
    // |payload_message|.
    const embedded_records_in_local_record = embedded_records[0].toRecords();
    assert_equals(
        embedded_records_in_local_record.length, 1,
        'Only one embedded record.');
    // The only one embedded record has correct content.
    assert_equals(
        embedded_records_in_local_record[0].recordType, 'text', 'recordType');
    assert_equals(
        embedded_records_in_local_record[0].mediaType, null, 'mediaType');
    assert_equals(embedded_records_in_local_record[0].id, test_record_id, 'id');
    const decoder = new TextDecoder();
    assert_equals(
        decoder.decode(embedded_records_in_local_record[0].data),
        test_text_data,
        'data has the same content with the original dictionary');
  }
}, 'NDEFRecord constructor with local record type');

test(() => {
  let buffer = new ArrayBuffer(4);
  new Uint8Array(buffer).set([1, 2, 3, 4]);
  const encoder = new TextEncoder();
  const uri_record = createUrlRecord(test_url_data);
  const title_record = createTextRecord(test_text_data, 'utf-8', 'en');
  const type_record = createRecord(':t', encoder.encode('image/gif'));
  const size_record = createRecord(':s', new Uint32Array([4096]));
  const action_record = createRecord(':act', new Uint8Array([3]));
  const icon_record = createRecord('mime', buffer, test_record_id, 'image/gif');
  const payload_message = createMessage([
    uri_record, title_record, type_record, size_record, action_record,
    icon_record
  ]);
  const smart_poster_record =
      createRecord('smart-poster', payload_message, 'dummy_record_id');

  const record = new NDEFRecord(smart_poster_record);
  assert_equals(record.recordType, 'smart-poster', 'recordType');
  assert_equals(record.mediaType, null, 'mediaType');
  assert_equals(record.id, 'dummy_record_id', 'id');
  const embedded_records = record.toRecords();
  assert_equals(embedded_records.length, 6, 'length');

  const decoder = new TextDecoder();
  let embedded_record_types = [];
  for (let record of embedded_records) {
    embedded_record_types.push(record.recordType);
    switch (record.recordType) {
      case 'url':
        assert_equals(record.mediaType, null, 'uri record\'s mediaType');
        assert_equals(record.id, test_record_id, 'uri record\'s id');
        assert_equals(
            decoder.decode(record.data), test_url_data, 'uri record\'s data');
        break;
      case 'text':
        assert_equals(record.mediaType, null, 'title record\'s mediaType');
        assert_equals(record.id, test_record_id, 'title record\'s id');
        assert_equals(record.encoding, 'utf-8', 'title record\'s encoding');
        assert_equals(record.lang, 'en', 'title record\'s lang');
        assert_equals(
            decoder.decode(record.data), test_text_data,
            'title record\'s data');
        break;
      case ':t':
        assert_equals(record.mediaType, null, 'type record\'s mediaType');
        assert_equals(record.id, null, 'type record\'s id');
        assert_equals(
            decoder.decode(record.data), 'image/gif', 'type record\'s data');
        break;
      case ':s':
        assert_equals(record.mediaType, null, 'size record\'s mediaType');
        assert_equals(record.id, null, 'size record\'s id');
        assert_equals(
            record.data.byteLength, 4, 'byteLength of size record\'s data');
        assert_equals(
            new Uint32Array(record.data.buffer)[0], 4096,
            'value of size record\'s data');
        break;
      case ':act':
        assert_equals(record.mediaType, null, 'action record\'s mediaType');
        assert_equals(record.id, null, 'action record\'s id');
        assert_equals(
            record.data.byteLength, 1, 'byteLength of action record\'s data');
        assert_equals(
            new Uint8Array(record.data.buffer)[0], 3,
            'value of action record\'s data');
        break;
      case 'mime':
        assert_equals(
            record.mediaType, 'image/gif', 'icon record\'s mediaType');
        assert_equals(record.id, test_record_id, 'icon record\'s id');
        assert_array_equals(
            new Uint8Array(record.data.buffer), [1, 2, 3, 4],
            'icon record\'s mediaType');
        break;
      default:
        assert_unreached('Unknown recordType');
    }
  }
  assert_array_equals(
      embedded_record_types.sort(), [':act', ':s', ':t', 'mime', 'text', 'url'],
      'smart-poster record\'s contained record types');
}, 'NDEFRecord constructor with smart-poster record type');

test(() => {
  const uri_record = createUrlRecord(test_url_data);
  const smart_poster_record = createRecord(
      'smart-poster', createMessage([uri_record]), 'dummy_record_id');
  const record = new NDEFRecord(smart_poster_record);
  assert_equals(record.recordType, 'smart-poster', 'recordType');
  assert_equals(record.mediaType, null, 'mediaType');
  assert_equals(record.id, 'dummy_record_id', 'id');
  const embedded_records = record.toRecords();

  // smart-poster record only contains a uri record.
  assert_equals(embedded_records.length, 1, 'length');
  const decoder = new TextDecoder();
  assert_equals(
      embedded_records[0].recordType, 'url', 'uri record\'s recordType');
  assert_equals(embedded_records[0].mediaType, null, 'uri record\'s mediaType');
  assert_equals(embedded_records[0].id, test_record_id, 'uri record\'s id');
  assert_equals(
      decoder.decode(embedded_records[0].data), test_url_data,
      'uri record\'s data');
}, 'NDEFRecord constructor with smart-poster record type that contains only a mandatory uri record');

test(() => {
  assert_throws_js(
      TypeError, () => new NDEFRecord(createRecord('EMptY')),
      'Unknown record type.');
  assert_throws_js(
      TypeError, () => new NDEFRecord(createRecord('TeXt', test_text_data)),
      'Unknown record type.');
  assert_throws_js(
      TypeError, () => new NDEFRecord(createRecord('uRL', test_url_data)),
      'Unknown record type.');
  assert_throws_js(
      TypeError, () => new NDEFRecord(createRecord('Mime', test_buffer_data)),
      'Unknown record type.');
  assert_throws_js(
      TypeError,
      () => new NDEFRecord(createRecord('sMart-PosTER', test_url_data)),
      'Unknown record type.');
}, 'NDEFRecord constructor with record type string being treated as case sensitive');

test(() => {
  assert_throws_js(
      TypeError,
      () => new NDEFRecord(createRecord('example.com:hellö', test_buffer_data)),
      'The external type must be an ASCII string.');

  // Length of the external type is 255, OK.
  const record = new NDEFRecord(createRecord(
      [...Array(251)].map(_ => 'a').join('') + ':xyz', test_buffer_data));
  // Exceeding 255, Throws.
  assert_throws_js(
      TypeError,
      () => new NDEFRecord(createRecord(
          [...Array(252)].map(_ => 'a').join('') + ':xyz', test_buffer_data)),
      'The external type should not be longer than 255.');

  assert_throws_js(
      TypeError, () => new NDEFRecord(createRecord('xyz', test_buffer_data)),
      'The external type must have a \':\'.');
  assert_throws_js(
      TypeError, () => new NDEFRecord(createRecord(':xyz', test_buffer_data)),
      'The domain should not be empty.');
  assert_throws_js(
      TypeError,
      () => new NDEFRecord(createRecord('example.com:', test_buffer_data)),
      'The type should not be empty.');
  assert_throws_js(
      TypeError,
      () => new NDEFRecord(createRecord('example.com:xyz[', test_buffer_data)),
      'The type should not contain \'[\'.');
  assert_throws_js(
      TypeError,
      () => new NDEFRecord(createRecord('example.com:xyz~', test_buffer_data)),
      'The type should not contain \'~\'.');
  assert_throws_js(
      TypeError,
      () => new NDEFRecord(createRecord('example.com:xyz/', test_buffer_data)),
      'The type should not contain \'/\'.');
}, 'NDEFRecord constructor with invalid external record type');

test(() => {
  const encoder = new TextEncoder();
  const uri_record = createUrlRecord(test_url_data);
  const title_record = createTextRecord(test_text_data, 'utf-8', 'en');
  const type_record = createRecord(':t', encoder.encode('image/gif'));
  const size_record = createRecord(':s', new Uint32Array([4096]));
  const action_record = createRecord(':act', new Uint8Array([0]));
  const icon_record =
      createRecord('mime', test_buffer_data, test_record_id, 'image/gif');

  const invalid_data_list = [
    {
      data: 'A string is not a NDEFMessageInit',
      message: 'A string is not allowed.'
    },
    {data: test_buffer_data, message: 'An ArrayBuffer is not allowed.'}, {
      data: createMessage(
          [title_record, type_record, size_record, action_record, icon_record]),
      message: 'Must contain a URI record.'
    },
    {
      data: createMessage([
        uri_record, title_record, type_record, size_record, action_record,
        icon_record, uri_record
      ]),
      message: 'Must not contain more than one uri record.'
    },
    {
      data: createMessage([
        uri_record, title_record, type_record, size_record, action_record,
        icon_record, type_record
      ]),
      message: 'Must not contain more than one type record.'
    },
    {
      data: createMessage([
        uri_record, title_record, type_record, size_record, action_record,
        icon_record, size_record
      ]),
      message: 'Must not contain more than one size record.'
    },
    {
      data: createMessage([
        uri_record, title_record, type_record, size_record, action_record,
        icon_record, action_record
      ]),
      message: 'Must not contain more than one action record.'
    },
    {
      data: createMessage([
        uri_record, title_record, type_record, action_record, icon_record,
        createRecord(':s', new Uint8Array([1]))
      ]),
      message:
          'Size record must have payload as 4-byte 32 bit unsigned integer.'
    },
    {
      data: createMessage([
        uri_record, title_record, type_record, size_record, icon_record,
        createRecord(':act', new Uint32Array([0]))
      ]),
      message:
          'Action record must have payload as 1-byte 8 bit unsigned integer.'
    }
  ];

  invalid_data_list.forEach(entry => {
    assert_throws_js(
        TypeError,
        () => new NDEFRecord(createRecord('smart-poster', entry.data)),
        entry.message);
  });
}, 'NDEFRecord constructor for smart-poster record with invalid embedded records.');

test(() => {
  assert_throws_js(
      TypeError, () => new NDEFRecord(createRecord(':xyz', test_buffer_data)),
      'The local type record must be embedded in the payload of another record (smart-poster, external, or local)');

  // The following test cases use an external type record embedding our target
  // local type record.

  const local_record = createRecord(':xyz', test_buffer_data);
  const payload_message = createMessage([local_record]);
  const external_record_embedding_local_record =
      createRecord('example.com:foo', payload_message);

  // OK.
  new NDEFRecord(external_record_embedding_local_record);
  local_record.recordType = ':xyZ123';
  new NDEFRecord(external_record_embedding_local_record);
  local_record.recordType = ':123XYz';
  new NDEFRecord(external_record_embedding_local_record);

  local_record.recordType = ':hellö';
  assert_throws_js(
      TypeError, () => new NDEFRecord(external_record_embedding_local_record),
      'The local type must be an ASCII string.');

  // Length of the local type excluding the prefix ':' is 255, OK.
  local_record.recordType = ':' + [...Array(255)].map(_ => 'a').join('');
  const record_255 = new NDEFRecord(external_record_embedding_local_record);

  // Exceeding 255, Throws.
  local_record.recordType = ':' + [...Array(256)].map(_ => 'a').join('');
  assert_throws_js(
      TypeError, () => new NDEFRecord(external_record_embedding_local_record),
      'The local type excluding the prefix \':\' should not be longer than 255.');

  local_record.recordType = 'xyz';
  assert_throws_js(
      TypeError, () => new NDEFRecord(external_record_embedding_local_record),
      'The local type must start with a \':\'.');

  local_record.recordType = ':Xyz';
  assert_throws_js(
      TypeError, () => new NDEFRecord(external_record_embedding_local_record),
      'The local type must have a lower case letter or digit following the prefix \':\'.');

  local_record.recordType = ':-xyz';
  assert_throws_js(
      TypeError, () => new NDEFRecord(external_record_embedding_local_record),
      'The local type must have a lower case letter or digit following the prefix \':\'.');
}, 'NDEFRecord constructor with various local record types');
