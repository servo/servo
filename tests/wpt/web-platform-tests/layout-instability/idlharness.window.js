// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://wicg.github.io/layout-instability/

'use strict';

idl_test(
  ['layout-instability'],
  ['performance-timeline'],
  idl_array => {
    idl_array.add_objects({
      // LayoutShift: [ TODO ]
    });
  }
);
