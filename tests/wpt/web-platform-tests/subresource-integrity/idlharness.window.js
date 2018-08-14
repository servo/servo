// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/webappsec-subresource-integrity/

'use strict';

idl_test(
  ['SRI'],
  ['html', 'cssom', 'dom'],
  idl_array => {
    idl_array.add_objects({
      HTMLScriptElement: ['document.createElement("script")'],
      HTMLLinkElement: ['document.createElement("link")'],
    });
  }
);
