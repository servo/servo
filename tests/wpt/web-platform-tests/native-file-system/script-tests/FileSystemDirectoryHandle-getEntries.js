directory_test(async (t, root) => {
  const file_name1 = 'foo1.txt';
  const file_name2 = 'foo2.txt';
  await createFileWithContents(t, file_name1, 'contents', /*parent=*/ root);
  await createFileWithContents(t, file_name2, 'contents', /*parent=*/ root);

  let abortIter = async (dir) => {
    for await (let entry of dir.getEntries()) {
      return entry.name;
    }
  };

  try {
    await abortIter(root);
  } catch(e) {
    assert_unreached('Error thrown on iteration abort.');
  }

}, 'getEntries(): returning early from an iteration works');

directory_test(async (t, root) => {
  const file_name1 = 'foo1.txt';
  const file_name2 = 'foo2.txt';
  await createFileWithContents(t, file_name1, 'contents', /*parent=*/ root);
  await createFileWithContents(t, file_name2, 'contents', /*parent=*/ root);

  let fullIter = async (dir) => {
    let name;
    for await (let entry of dir.getEntries()) {
      name = entry.name;
    }
    return name;
  };

  try {
    await fullIter(root);
  } catch(e) {
    assert_unreached('Error thrown on iteration.');
  }

}, 'getEntries(): full iteration works');
