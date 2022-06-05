// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

idl_test(
  ['file-system-access'],
  ['fs', 'permissions', 'html', 'dom'],
  idl_array => {
    if (self.GLOBAL.isWindow()) {
      idl_array.add_objects({
        Window: ['window'],
        // TODO: DataTransferItem
      });
    }
  }
);
