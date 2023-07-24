// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

"use strict";

idl_test(
  ['webidl'],
  [],
  idl_array => {
    idl_array.add_objects({
      DOMException: ['new DOMException()',
                     'new DOMException("my message")',
                     'new DOMException("my message", "myName")']
    });
  }
);
