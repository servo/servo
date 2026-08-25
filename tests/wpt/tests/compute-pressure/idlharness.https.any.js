// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: global=window,dedicatedworker,sharedworker

// https://w3c.github.io/compute-pressure/

'use strict';

idl_test(['compute-pressure'], ['dom', 'html'], async idl_array => {
  idl_array.add_objects({
    PressureObserver: ['observer'],
  });

  self.observer = new PressureObserver(() => {});
});
