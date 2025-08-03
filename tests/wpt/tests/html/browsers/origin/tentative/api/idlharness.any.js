// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

idl_test(
  ['origin.tentative'], [], (idl_array) => {
    idl_array.add_objects({
      Origin: ["new Origin()"],
    });
  });

