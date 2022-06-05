// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

idl_test(
  ['sanitizer-api.tentative'],
  ['html'],
  idl_array => {
    idl_array.add_objects({
      Sanitizer: ['new Sanitizer({})']
    });
  }
);
