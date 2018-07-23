// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/selection-api/

idl_test(
  ['selection-api'],
  ['dom', 'html'],
  idlArray => {
    idlArray.add_objects({
      Window: ['window'],
      Document: ['document'],
      Selection: ['getSelection()'],
      GlobalEventHandlers: ['self'],
    });
  },
  'selection-api interfaces'
);
