// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

// https://w3c.github.io/web-nfc/

const record = {
  recordType: "text",
  data: "Hello World",
  id: "/custom/path"
};
const message = {
  url: "/custom/path",
  records: [record]
};

idl_test(
  ['web-nfc'],
  ['html', 'dom', 'webidl'],
  idl_array => {
    idl_array.add_objects({
      NDEFReader: ['new NDEFReader();'],
      NDEFRecord: [`new NDEFRecord(${JSON.stringify(record)});`],
      NDEFMessage: [`new NDEFMessage(${JSON.stringify(message)});`],
      NDEFReadingEvent: [`new NDEFReadingEvent("reading", { message: ${JSON.stringify(message)} })`],
    });
  }
);
