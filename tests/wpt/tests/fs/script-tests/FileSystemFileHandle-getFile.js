'use strict';

directory_test(async (t, root) => {
  const fileContents = 'awesome content';
  let handle =
      await createFileWithContents('foo.txt', fileContents, /*parent=*/ root);
  let file = await handle.getFile();
  let slice = file.slice(1, file.size);
  let actualContents = await slice.text();
  assert_equals(actualContents, fileContents.slice(1, fileContents.length));
}, 'getFile() provides a file that can be sliced');

directory_test(async (t, root) => {
  const handle = await createEmptyFile('mtime.txt', root);
  let file = await handle.getFile();
  const first_mtime = file.lastModified;

  // We wait for 2s here to ensure that the files do not have the
  // same modification time. Some filesystems have low resolutions
  // for modification timestamps.
  let timeout = new Promise(resolve => {
    t.step_timeout(resolve, 2000);
  });
  await timeout;

  const writer = await handle.createWritable({keepExistingData: false});
  await writer.write(new Blob(['foo']));
  await writer.close();

  file = await handle.getFile();
  const second_mtime = file.lastModified;

  // We wait for 5 ms here to ensure that `lastModified`
  // from the File objects is stable between getFile invocations.
  timeout = new Promise(resolve => {
    t.step_timeout(resolve, 5);
  });
  await timeout;
  let fileReplica = await handle.getFile();
  assert_equals(second_mtime, fileReplica.lastModified);

  assert_less_than(first_mtime, second_mtime);
}, 'getFile() returns last modified time');

directory_test(async (t, root) => {
  const fileName = "fileAttributesTest.txt";

  const fileHandle = await createEmptyFile(fileName, root);
  assert_equals(fileHandle.name, fileName);

  const file = await fileHandle.getFile();
  assert_equals(file.name, fileName);
}, 'getFile() returns expected name');
