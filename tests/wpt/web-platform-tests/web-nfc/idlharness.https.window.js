// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/web-nfc/

const record = {
  recordType: "text",
  mediaType: "text/plain",
  data: "Hello World",
  id: "/custom/path"
};
const message = {
  url: "/custom/path",
  records: [record]
};

idl_test(
  ['web-nfc'],
  ['html', 'dom', 'WebIDL'],
  idl_array => {
    idl_array.add_objects({
      NFCWriter: ['new NFCWriter();'],
      NFCReader: ['new NFCReader();'],
      NDEFRecord: [`new NDEFRecord(${JSON.stringify(record)});`],
      NDEFMessage: [`new NDEFMessage(${JSON.stringify(message)});`],
      NFCReadingEvent: [`new NFCReadingEvent("reading", { message: ${JSON.stringify(message)} })`],
      NFCErrorEvent: ['new NFCErrorEvent("error", { error: new DOMException() });'],
    });
  }
);
