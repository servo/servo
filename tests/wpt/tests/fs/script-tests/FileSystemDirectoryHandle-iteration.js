'use strict';

directory_test(async (t, root) => {
  const file_name1 = 'foo1.txt';
  const file_name2 = 'foo2.txt';
  await createFileWithContents(file_name1, 'contents', /*parent=*/ root);
  await createFileWithContents(file_name2, 'contents', /*parent=*/ root);

  for await (let entry of root) {
    break;
  }

}, 'returning early from an iteration doesn\'t crash');

directory_test(async (t, root) => {
  const file_name1 = 'foo1.txt';
  const file_name2 = 'foo2.txt';
  await createFileWithContents(file_name1, 'contents', /*parent=*/ root);
  await createFileWithContents(file_name2, 'contents', /*parent=*/ root);

  let names = [];
  for await (let entry of root) {
    assert_true(Array.isArray(entry));
    assert_equals(entry.length, 2);
    assert_equals(typeof entry[0], 'string');
    assert_true(entry[1] instanceof FileSystemFileHandle);
    assert_equals(entry[0], entry[1].name);
    names.push(entry[0]);
  }
  names.sort();
  assert_array_equals(names, [file_name1, file_name2]);

}, '@@asyncIterator: full iteration works');

directory_test(async (t, root) => {
  const file_name1 = 'foo1.txt';
  const file_name2 = 'foo2.txt';
  await createFileWithContents(file_name1, 'contents', /*parent=*/ root);
  await createFileWithContents(file_name2, 'contents', /*parent=*/ root);

  let names = [];
  for await (let entry of root.entries()) {
    assert_true(Array.isArray(entry));
    assert_equals(entry.length, 2);
    assert_equals(typeof entry[0], 'string');
    assert_true(entry[1] instanceof FileSystemFileHandle);
    assert_equals(entry[0], entry[1].name);
    names.push(entry[0]);
  }
  names.sort();
  assert_array_equals(names, [file_name1, file_name2]);
}, 'entries: full iteration works');

directory_test(async (t, root) => {
  const file_name1 = 'foo1.txt';
  const file_name2 = 'foo2.txt';
  await createFileWithContents(file_name1, 'contents', /*parent=*/ root);
  await createFileWithContents(file_name2, 'contents', /*parent=*/ root);

  let names = [];
  for await (let entry of root.values()) {
    assert_true(entry instanceof FileSystemFileHandle);
    names.push(entry.name);
  }
  names.sort();
  assert_array_equals(names, [file_name1, file_name2]);
}, 'values: full iteration works');

directory_test(async (t, root) => {
  const file_name1 = 'foo1.txt';
  const file_name2 = 'foo2.txt';
  await createFileWithContents(file_name1, 'contents', /*parent=*/ root);
  await createFileWithContents(file_name2, 'contents', /*parent=*/ root);

  let names = [];
  for await (let entry of root.keys()) {
    assert_equals(typeof entry, 'string');
    names.push(entry);
  }
  names.sort();
  assert_array_equals(names, [file_name1, file_name2]);
}, 'keys: full iteration works');

directory_test(async (t, root) => {
  const file_name1 = 'foo1.txt';
  await createFileWithContents(file_name1, 'contents', /*parent=*/ root);

  const next = (() => {
    const iterator = root.entries();
    return iterator.next();
  })();
  garbageCollect();
  let entry = await next;
  assert_false(entry.done);
  assert_true(Array.isArray(entry.value));
  assert_equals(entry.value.length, 2);
  assert_equals(entry.value[0], file_name1);
  assert_true(entry.value[1] instanceof FileSystemFileHandle);
  assert_equals(entry.value[1].name, file_name1);
}, 'iteration while iterator gets garbage collected');
