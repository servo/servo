// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/selection-api/

idl_test(
  ['selection-api'],
  ['html', 'dom'],
  idlArray => {
    idlArray.add_objects({
      Window: ['window'],
      Document: ['document'],
      Selection: ['getSelection()'],
    });
  }
);
