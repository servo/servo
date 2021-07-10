// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/webappsec-csp/embedded/

'use strict';

idl_test(
  ['csp-embedded-enforcement'],
  ['html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      HTMLIFrameElement: ['document.createElement("iframe")'],
    });
  }
);
