// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/DOM-Parsing/

idl_test(
  ['DOM-Parsing'],
  ['dom'],
  idlArray => {
    idlArray.add_objects({
      Element: ['document.createElement("div")'],
      Range: ['new Range()'],
      XMLSerializer: ['new XMLSerializer()'],
    })
  }
);
