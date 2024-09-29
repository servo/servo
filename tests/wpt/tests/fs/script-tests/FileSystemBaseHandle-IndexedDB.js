'use strict';

directory_test(async (t, root_dir) => {
  const handles = await create_file_system_handles(root_dir);

  const db = await createDatabase(t, db => {
    const store = db.createObjectStore('store');
  });

  const value = handles;

  const tx = db.transaction('store', 'readwrite');
  const store = tx.objectStore('store');
  await promiseForRequest(t, store.put(value, 'key'));
  const result = await promiseForRequest(t, store.get('key'));

  await promiseForTransaction(t, tx);

  assert_true(Array.isArray(result), 'Result should be an array');
  assert_equals(result.length, value.length);
  await assert_equals_cloned_handles(result, value);
}, 'Store handle in IndexedDB and read from pending transaction.');

directory_test(async (t, root_dir) => {
  const handles = await create_file_system_handles(root_dir);

  const db = await createDatabase(t, db => {
    const store = db.createObjectStore('store');
  });

  const value = handles;

  let tx = db.transaction('store', 'readwrite');
  let store = tx.objectStore('store');
  await promiseForRequest(t, store.put(value, 'key'));
  await promiseForTransaction(t, tx);

  tx = db.transaction('store', 'readonly');
  store = tx.objectStore('store');
  const result = await promiseForRequest(t, store.get('key'));
  await promiseForTransaction(t, tx);

  assert_true(Array.isArray(result), 'Result should be an array');
  assert_equals(result.length, value.length);
  await assert_equals_cloned_handles(result, value);
}, 'Store handle in IndexedDB and read from new transaction.');

directory_test(async (t, root_dir) => {
  const handles = await create_file_system_handles(root_dir);

  const db = await createDatabase(t, db => {
    const store = db.createObjectStore('store');
  });

  const value = {handles, blob: new Blob(['foobar'])};

  let tx = db.transaction('store', 'readwrite');
  let store = tx.objectStore('store');
  await promiseForRequest(t, store.put(value, 'key'));
  await promiseForTransaction(t, tx);

  tx = db.transaction('store', 'readonly');
  store = tx.objectStore('store');
  const result = await promiseForRequest(t, store.get('key'));
  await promiseForTransaction(t, tx);

  assert_true(Array.isArray(result.handles), 'Result should be an array');
  assert_equals(result.handles.length, value.handles.length);
  await assert_equals_cloned_handles(result.handles, value.handles);

  assert_equals(await result.blob.text(), await value.blob.text());
}, 'Store handles and blobs in IndexedDB.');

directory_test(async (t, root_dir) => {
  const handles = await create_file_system_handles(root_dir);

  const db = await createDatabase(t, db => {
    const store = db.createObjectStore('store');
  });

  const value = handles;

  let tx = db.transaction('store', 'readwrite');
  let store = tx.objectStore('store');
  await promiseForRequest(t, store.put(value, 'key'));
  await promiseForTransaction(t, tx);

  tx = db.transaction('store', 'readonly');
  store = tx.objectStore('store');
  let cursor_request = store.openCursor();
  await requestWatcher(t, cursor_request).wait_for('success');
  const result = cursor_request.result.value;
  await promiseForTransaction(t, tx);

  assert_true(Array.isArray(result), 'Result should be an array');
  assert_equals(result.length, value.length);
  await assert_equals_cloned_handles(result, value);
}, 'Store handle in IndexedDB and read using a cursor.');

directory_test(async (t, root_dir) => {
  const handles = await create_file_system_handles(root_dir);

  const db = await createDatabase(t, db => {
    const store = db.createObjectStore('store', {keyPath: 'key'});
  });

  const value = handles;
  let tx = db.transaction('store', 'readwrite');
  let store = tx.objectStore('store');
  await promiseForRequest(t, store.put({key: 'key', value}));
  await promiseForTransaction(t, tx);

  tx = db.transaction('store', 'readonly');
  store = tx.objectStore('store');
  const result = await promiseForRequest(t, store.get('key'));
  await promiseForTransaction(t, tx);

  assert_true(Array.isArray(result.value), 'Result should be an array');
  assert_equals(result.value.length, value.length);
  await assert_equals_cloned_handles(result.value, value);
}, 'Store handle in IndexedDB using inline keys.');

directory_test(async (t, root_dir) => {
  const expected_root_name = root_dir.name;

  const db = await createDatabase(t, db => {
    const store = db.createObjectStore('store', {keyPath: 'key'});
  });

  const value = [ root_dir ];
  let tx = db.transaction('store', 'readwrite');
  let store = tx.objectStore('store');
  await promiseForRequest(t, store.put({key: 'key', value}));
  await promiseForTransaction(t, tx);

  tx = db.transaction('store', 'readonly');
  store = tx.objectStore('store');
  const result = await promiseForRequest(t, store.get('key'));
  await promiseForTransaction(t, tx);

  const actual = result.value[ 0 ];
  assert_equals(actual.name, expected_root_name);
  assert_true(await root_dir.isSameEntry(actual));
}, 'Store and retrieve the root directory from IndexedDB.');
