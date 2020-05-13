// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

idl_test(
  ['visual-viewport'],
  ['html', 'dom'],
  idl_array => {
    idl_array.add_objects({
        VisualViewport: ['self.visualViewport'],
        Window: ['self'],
    });
  }
);
