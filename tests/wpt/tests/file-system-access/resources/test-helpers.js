// A special path component meaning "this directory."
const kCurrentDirectory = '.';

// A special path component meaning "the parent directory."
const kParentDirectory = '..';

// Array of separators used to separate components in hierarchical paths.
let kPathSeparators;
if (navigator.userAgent.includes('Windows NT')) {
  // Windows uses both '/' and '\' as path separators.
  kPathSeparators = ['/', '\\'];
} else {
  kPathSeparators = ['/'];
}

async function getFileSize(handle) {
  const file = await handle.getFile();
  return file.size;
}

async function getFileContents(handle) {
  const file = await handle.getFile();
  return new Response(file).text();
}

async function getDirectoryEntryCount(handle) {
  let result = 0;
  for await (let entry of handle) {
    result++;
  }
  return result;
}

async function getSortedDirectoryEntries(handle) {
  let result = [];
  for await (let entry of handle.values()) {
    if (entry.kind === 'directory')
      result.push(entry.name + '/');
    else
      result.push(entry.name);
  }
  result.sort();
  return result;
}

async function createDirectory(test, name, parent) {
  const new_dir_handle = await parent.getDirectoryHandle(name, {create: true});
  test.add_cleanup(async () => {
    try {
      await parent.removeEntry(name, {recursive: true});
    } catch (e) {
      // Ignore any errors when removing directories, as tests might
      // have already removed the directory.
    }
  });
  return new_dir_handle;
}

async function createEmptyFile(test, name, parent) {
  const handle = await parent.getFileHandle(name, {create: true});
  test.add_cleanup(async () => {
    try {
      await parent.removeEntry(name);
    } catch (e) {
      // Ignore any errors when removing files, as tests might already remove
      // the file.
    }
  });
  // Make sure the file is empty.
  assert_equals(await getFileSize(handle), 0);
  return handle;
}

async function createFileWithContents(test, name, contents, parent) {
  const handle = await createEmptyFile(test, name, parent);
  const writer = await handle.createWritable();
  await writer.write(new Blob([contents]));
  await writer.close();
  return handle;
}
