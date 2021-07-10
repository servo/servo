// META: global=window
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

// https://wicg.github.io/get-installed-related-apps/spec/

idl_test(
  ['get-installed-related-apps'],
  ['html'],
  idl_array => {
    idl_array.add_objects({
      Navigator: ['navigator'],
    });
  }
)
