// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

// https://wicg.github.io/element-timing/

'use strict';

idl_test(
  ['element-timing'],
  ['performance-timeline', 'dom'],
  idl_array => {
    idl_array.add_objects({
      // PerformanceElementTiming: [ TODO ]
    });
  }
);
