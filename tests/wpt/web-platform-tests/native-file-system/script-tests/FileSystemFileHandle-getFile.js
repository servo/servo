directory_test(async (t, root) => {
  const fileContents = 'awesome content';
  let handle = await createFileWithContents(t, 'foo.txt', fileContents, /*parent=*/ root);
  let file = await handle.getFile();
  let slice = file.slice(1, file.size);
  let actualContents = await slice.text();
  assert_equals(actualContents, fileContents.slice(1, fileContents.length));
}, 'getFile() provides a file that can be sliced');
