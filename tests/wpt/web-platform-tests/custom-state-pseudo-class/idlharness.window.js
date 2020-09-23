// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

idl_test(
  ['custom-state-pseudo-class'],
  ['html', 'wai-aria'],
  idl_array => {
    idl_array.add_objects({
      // Nothing to add; spec only defined a partial interface.
    });
  }
);
