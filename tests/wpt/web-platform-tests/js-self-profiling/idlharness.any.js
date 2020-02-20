// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

idl_test(
  ['js-self-profiling'],
  ['hr-time', 'dom'],
  async idl_array => {
    idl_array.add_objects({
      Performance: ['performance'],
      Profiler: ['profiler'],
    });

    self.profiler = await performance.profile({
      sampleInterval: 1,
    });
  }
);
