// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://wicg.github.io/savedata/

idl_test(
  ['savedata'],
  ['netinfo', 'html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      NetworkInformation: ['navigator.connection']
    });
  }
);
