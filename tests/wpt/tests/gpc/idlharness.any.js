// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

idl_test(
  ['gpc'],
  ['html'],
  idl_array => {
    if (self.Window) {
      idl_array.add_objects({ Navigator: ['navigator'] });
    } else {
      idl_array.add_objects({ WorkerNavigator: ['navigator'] });
    }
  }
);
