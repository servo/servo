// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/web-nfc/
const message = {
  url: "/custom/path",
  records: [{
    recordType: "text",
    data: "Hello World"
  }]
}

idl_test(
  ['web-nfc'],
  ['html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      NFCWriter: ['new NFCWriter();'],
      NFCReader: ['new NFCReader();'],
      NFCReadingEvent: [`new NFCReadingEvent("reading", { message: ${JSON.stringify(message)} })`],
      NFCErrorEvent: ['new NFCErrorEvent("error", { error: new DOMException() });'],
    });
  }
);
