// META: title=formData.set(blob) and formData.set(file)

"use strict";

const formData = new FormData();

function assert_file_identity(formData, name, expected) {
  assert_equals(formData.get(name), expected,
                'get() should return the same File');
  assert_equals(formData.getAll(name)[0], expected,
                'getAll() should return the same File');
  const entry = Array.from(formData).find(([entryName]) => entryName === name);
  assert_equals(entry[1], expected, 'the iterator should return the same File');
}

test(() => {
  const value = new Blob();
  formData.set("blob-1", value);
  const blob1 = formData.get("blob-1");
  assert_not_equals(blob1, value);
  assert_equals(blob1.constructor.name, "File");
  assert_equals(blob1.name, "blob");
  assert_equals(blob1.type, "");
  assert_file_identity(formData, 'blob-1', blob1);
  assert_less_than(Math.abs(blob1.lastModified - Date.now()), 200, "lastModified should be now");
}, "blob without type");

test(() => {
  const value = new Blob([], { type: "text/plain" });
  formData.set("blob-2", value);
  const blob2 = formData.get("blob-2");
  assert_not_equals(blob2, value);
  assert_equals(blob2.constructor.name, "File");
  assert_equals(blob2.name, "blob");
  assert_equals(blob2.type, "text/plain");
  assert_less_than(Math.abs(blob2.lastModified - Date.now()), 200, "lastModified should be now");
}, "blob with type");

test(() => {
  const value = new Blob();
  formData.set("blob-3", value, "custom name");
  const blob3 = formData.get("blob-3");
  assert_not_equals(blob3, value);
  assert_equals(blob3.constructor.name, "File");
  assert_equals(blob3.name, "custom name");
  assert_equals(blob3.type, "");
  assert_file_identity(formData, 'blob-3', blob3);
  assert_less_than(Math.abs(blob3.lastModified - Date.now()), 200, "lastModified should be now");
}, "blob with custom name");

test(() => {
  const value = new File([], "name");
  formData.set("file-1", value);
  const file1 = formData.get("file-1");
  assert_equals(file1, value);
  assert_equals(file1.constructor.name, "File");
  assert_equals(file1.name, "name");
  assert_equals(file1.type, "");
  assert_file_identity(formData, 'file-1', value);
  assert_less_than(Math.abs(file1.lastModified - Date.now()), 200, "lastModified should be now");
}, "file without lastModified or custom name");

test(() => {
  const value = new File([], "name", { lastModified: 123 });
  formData.set("file-2", value, "custom name");
  const file2 = formData.get("file-2");
  assert_not_equals(file2, value);
  assert_equals(file2.constructor.name, "File");
  assert_equals(file2.name, "custom name");
  assert_equals(file2.type, "");
  assert_equals(file2.lastModified, 123, "lastModified should be 123");
  assert_file_identity(formData, 'file-2', file2);
}, "file with lastModified and custom name");

test(() => {
  const appendFormData = new FormData();
  appendFormData.append('blob', new Blob());
  const file = appendFormData.get('blob');
  assert_file_identity(appendFormData, 'blob', file);

  appendFormData.append('custom', new Blob(), 'custom name');
  const customFile = appendFormData.get('custom');
  assert_equals(customFile.name, 'custom name');
  assert_file_identity(appendFormData, 'custom', customFile);
}, 'append() should create a single File for each Blob');
