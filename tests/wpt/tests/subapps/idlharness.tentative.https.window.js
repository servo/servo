// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

idl_test(
  ['sub-apps.tentative'],
  ['html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      Navigator: ['navigator'],
      SubApps: ['navigator.subApps'],
    });
  }
);
