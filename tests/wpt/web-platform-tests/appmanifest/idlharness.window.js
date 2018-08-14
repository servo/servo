// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/manifest/

'use strict';

idl_test(
  ['appmanifest'],
  ['html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      Window: ['window'],
      BeforeInstallPromptEvent: ['new BeforeInstallPromptEvent("type")'],
    });
  }
);
