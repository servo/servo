// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://wicg.github.io/event-timing/

'use strict';

idl_test(
  ['event-timing'],
  ['performance-timeline', 'hr-time', 'dom'],
  idl_array => {
    idl_array.add_objects({
      Performance: ['performance'],
      // PerformanceEventTiming: [ TODO ]
    });
  }
);
