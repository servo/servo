// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=support.js

'use strict';

let resolve;
let globalItem;
let globalEntry;

let entriesPromise = new Promise(r => {
  resolve = r;
});

entry_test((t, entry, item) => {
  assert_true(entry.isDirectory);
  resolve(getEntriesAsPromise(entry));
  globalItem = item;
  globalEntry = entry;
  t.done();
});

idl_test(
  ['entries-api'],
  ['FileAPI', 'html', 'dom'],
  async idl_array => {
    const entries = await entriesPromise;
    window.samples = {
      item: globalItem,
      dirEntry: entries.filter(entry => entry.isDirectory)[0],
      fileEntry: entries.filter(entry => entry.isFile)[0],
      fileSystem: globalEntry.filesystem,
    };

    idl_array.add_objects({
      File: ['new File([], "example.txt")'],
      HTMLInputElement: ['document.createElement("input")'],
      DataTransferItem: ['samples.item'],
      FileSystemEntry: [],
      FileSystemDirectoryEntry: ['samples.dirEntry'],
      FileSystemDirectoryReader: ['samples.dirEntry.createReader()'],
      FileSystemFileEntry: ['samples.fileEntry'],
      FileSystem: ['samples.fileSystem'],
    });
  }
);
