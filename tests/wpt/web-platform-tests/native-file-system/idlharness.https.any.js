// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

idl_test(
  ['native-file-system'],
  ['html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      // TODO: Add instances of FileSystemHandle, FileSystemFileHandle,
      // FileSystemDirectoryHandle and FileSystemWriter.
    });
  }
);
