// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://wicg.github.io/is-input-pending/

idl_test(
  ['is-input-pending'],
  ['html', 'dom'],
  async idl_array => {
    idl_array.add_objects({
      Scheduling: ['navigator.scheduling'],
    });
  }
);
