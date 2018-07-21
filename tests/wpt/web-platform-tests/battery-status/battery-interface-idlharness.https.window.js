// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/battery/

'use strict';

idl_test(
  ['battery'],
  ['dom', 'html'],
  async idl_array => {
    idl_array.add_objects({
      Navigator: ['navigator'],
      BatteryManager: ['manager'],
    })

    self.manager = await navigator.getBattery();
  },
  'Test IDL implementation of Battery Status API'
);
