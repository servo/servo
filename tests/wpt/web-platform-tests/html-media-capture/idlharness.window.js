// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/html-media-capture/

idl_test(
  ['html-media-capture'],
  ['html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      HTMLInputElement: ['input'],
    });

    self.input = document.createElement('input');
  }
);
