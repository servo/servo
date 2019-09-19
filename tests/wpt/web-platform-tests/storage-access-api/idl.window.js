// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
'use strict';

const idl = `
  [OverrideBuiltins]
  partial interface Document {
    Promise<bool> hasStorageAccess();
    Promise<void> requestStorageAccess();
  };`;

idl_test(
  // Since we are testing a locally defined idl we'll manually manage our deps below.
  [],
  [],
  idl_array => {
    return fetch_spec('dom').then(function(idl_text) {
      idl_array.add_idls(idl);
      idl_array.add_dependency_idls(idl_text);
    });
  }
);
