// ----------------------------------------
// Test Utilities
// ----------------------------------------

setup({explicit_timeout: true});

const tests = [];
window.addEventListener('DOMContentLoaded', e => {
  const header = document.createElement('h1');
  header.innerText = document.title;
  document.body.appendChild(header);
  const elem = document.createElement('div');
  elem.style.cssText = 'height: 50px; border: 1px dotted red;';
  elem.innerHTML = 'Drop or paste the <b>support/upload</b> directory here.</div>';
  document.body.appendChild(elem);
  elem.addEventListener('dragover', e => {
    e.preventDefault();
  });
  const onDropOrPaste = dataTransfer => {
    for (let i = 0; i < dataTransfer.items.length; ++i) {
      const item = dataTransfer.items[i];
      if (item.kind !== 'file')
        continue;
      const entry = item.webkitGetAsEntry();
      elem.parentElement.removeChild(elem);
      tests.forEach(f => f(entry, item));
      break;
    }
  };
  elem.addEventListener('drop', e => {
    e.preventDefault();
    onDropOrPaste(e.dataTransfer);
  });
  elem.addEventListener('paste', e => {
    e.preventDefault();
    onDropOrPaste(e.clipboardData);
  });
});


// Registers a test to be run when an entry is dropped. Calls |func|
// with (test, entry, item); |func| must call `test.done()` when complete.
function entry_test(func, description) {
  const test = async_test(description);
  tests.push(test.step_func((entry, item) => func(test, entry, item)));
}

// Registers a test to be run when an entry is dropped. Digs the named
// |file| out of the dropped entry and calls |func| with
// (test, file_entry); |func| must call `test.done()` when complete.
function file_entry_test(name, func, description) {
  return entry_test((t, entry, item) => {
    getChildEntry(entry, name,
                  t.step_func((entry) => func(t, entry)),
                  t.unreached_func('Did not find expected file: ' + name));
  }, description);
}


// ----------------------------------------
// Paths
// ----------------------------------------

const INVALID_PATHS = [
  '\x00', 'a-\x00-b',
  '\\', 'a-\\-b'
];
const EMPTY_PATHS = ['', null, undefined];
const NOT_FOUND_PATHS = [
  'nope',
  '/upload/nope',
  './nope',
  'subdir/../nope',
  '\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f',
  '\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1a\x1b\x1c\x1d\x1e\x1f',
];

const DIR_PATHS = [
  'subdir',
  '/upload/subdir',
  './subdir',
  'subdir/.',
  'subdir/../subdir',
  'subdir/./../subdir',
  'subdir/../subdir/.',
  '//upload/subdir',
  '/upload//subdir',
  './/subdir',
  'subdir//.',
];
const FILE_PATHS = [
  'file.txt',
  '/upload/file.txt',
  'subdir/../file.txt',
  '//upload/file.txt',
  '/upload//file.txt',
  'subdir/./../file.txt',
];

// ----------------------------------------
// Helpers
// ----------------------------------------

// Wrapper for FileSystemDirectoryReader that yields all entries via a
// Promise.

function getEntriesAsPromise(dirEntry) {
  return new Promise((resolve, reject) => {
    const result = [];
    const reader = dirEntry.createReader();
    const doBatch = () => {
      reader.readEntries(entries => {
        if (entries.length > 0) {
          entries.forEach(e => result.push(e));
          doBatch();
        } else {
          resolve(result);
        }
      }, reject);
    };
    doBatch();
  });
}


// Wrapper for FileSystemDirectoryReader that yields a single entry by
// name via a callback. Can be used instead of getFile() or
// getDirectory() since not all implementations support those.

function getChildEntry(dirEntry, name, callback, errback) {
  getEntriesAsPromise(dirEntry)
    .then(entries => {
      const entry = entries.filter(entry => entry.name === name)[0];
      if (!entry)
        throw new Error('No such file: ' + name);
      return entry;
    }).then(callback, errback);
}
