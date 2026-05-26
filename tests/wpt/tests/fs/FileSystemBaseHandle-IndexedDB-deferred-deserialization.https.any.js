// META: script=resources/test-helpers.js
// META: script=resources/sandboxed-fs-test-helpers.js
// META: script=resources/messaging-helpers.js
// META: script=/IndexedDB/resources/support-promises.js

'use strict';

directory_test(async (t, root_dir) => {
  const handle = await createFileWithContents('file', 'contents', root_dir);

  const db = await createDatabase(t, db => {
    db.createObjectStore('store');
  });

  let tx = db.transaction('store', 'readwrite');
  await promiseForRequest(t, tx.objectStore('store').put(handle, 'key'));
  await promiseForTransaction(t, tx);

  tx = db.transaction('store', 'readwrite');
  const store = tx.objectStore('store');
  const getRequest = store.get('key');
  await requestWatcher(t, getRequest).wait_for('success');

  await promiseForRequest(t, store.delete('key'));
  await promiseForTransaction(t, tx);

  const retrieved = getRequest.result;
  assert_true(await handle.isSameEntry(retrieved));
  const file = await retrieved.getFile();
  assert_equals(await file.text(), 'contents');
}, 'Reading request.result after the record is deleted still produces a usable handle.');

directory_test(async (t, root_dir) => {
  const handle = await createFileWithContents('file', 'contents', root_dir);

  const db = await createDatabase(t, db => {
    db.createObjectStore('store');
  });

  const tx = db.transaction('store', 'readwrite');
  await promiseForRequest(t, tx.objectStore('store').put(handle, 'key'));
  await promiseForTransaction(t, tx);

  const readTx = db.transaction('store', 'readonly');
  const getRequest = readTx.objectStore('store').get('key');
  await requestWatcher(t, getRequest).wait_for('success');
  await promiseForTransaction(t, readTx);

  db.close();

  const retrieved = getRequest.result;
  assert_true(await handle.isSameEntry(retrieved));
  const file = await retrieved.getFile();
  assert_equals(await file.text(), 'contents');
}, 'Reading request.result after the database is closed still produces a usable handle.');
