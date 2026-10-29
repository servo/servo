// Helpers for Entries API tests that use a script-created DataTransfer, so no
// drag-and-drop or file picker interaction is required.

// Returns the DataTransferItem for a script-created file.
function createDataTransferItem() {
  const file = new File(['hello world'], 'test.txt', { type: 'text/plain' });
  const dataTransfer = new DataTransfer();
  dataTransfer.items.add(file);
  return dataTransfer.items[0];
}

// Returns the FileSystemFileEntry for a script-created file.
function createFileEntry() {
  return createDataTransferItem().webkitGetAsEntry();
}

// Returns the root FileSystemDirectoryEntry of a script-created file entry.
function createRoot() {
  return createFileEntry().filesystem.root;
}

// Reads all entries from a FileSystemDirectoryReader, batching until empty.
function readAll(reader) {
  return new Promise((resolve, reject) => {
    const result = [];
    const doBatch = () => {
      reader.readEntries(entries => {
        if (entries.length > 0) {
          result.push(...entries);
          doBatch();
        } else {
          resolve(result);
        }
      }, reject);
    };
    doBatch();
  });
}
